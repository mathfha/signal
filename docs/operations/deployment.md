# Deployment and operations

## Current runtime scope

The application currently fetches configured RSS feeds at startup and serves an
in-memory snapshot. It does not yet use SQLite or schedule recurring fetches.
The `/health` endpoint is a liveness check for the web process; it returns HTTP
200 even when an individual feed fails. A feed failure is shown on the page and
logged, but does not make the whole service unhealthy.

## Docker Compose

Docker Compose builds a release binary in a Rust builder image and runs it in a
Debian slim image as an unprivileged user. The container binds to
`0.0.0.0:3000`, while Compose publishes it only on the host's loopback address.
Use a reverse proxy on the same host for access from other machines.

Start or rebuild the service from the project directory:

```sh
docker compose up --build --detach
docker compose ps
docker compose logs --follow techpulse
```

Check the service from the host:

```sh
curl --fail http://127.0.0.1:3000/health
```

Docker checks `/health` every 30 seconds, with a 60-second startup grace
period. Inspect health state with `docker compose ps`. Update feed or category
configuration by rebuilding the image with `docker compose up --build --detach`.
Stop the service with `docker compose down`.

The Compose restart policy is `unless-stopped`: Docker restarts the container
after a process failure and when the Docker daemon starts, unless an operator
explicitly stopped it. A routine timed restart is not required. If a host
maintenance policy requires one, schedule it in the host's maintenance window;
for example, this cron entry restarts the service every Monday at 04:00 in the
host's local time (adjust the time and project path to the host):

```cron
0 4 * * 1 cd /srv/techpulse && docker compose restart techpulse
```

## Health monitoring

Monitor `GET /health` and alert when it does not return HTTP 200 or cannot be
reached. The endpoint reports whether the web process is responding. It does
not verify that RSS sources are reachable, and it cannot report database status
because SQLite persistence has not been added yet. Review `docker compose logs`
and the feed failure notice on the web page to investigate source problems.

## SQLite backups

There is no SQLite database file to back up in the current implementation. The
application stores only an in-memory feed snapshot, which is rebuilt when it
starts. Do not treat the container filesystem as persistent storage.

When SQLite storage is implemented, set a database path on a persistent host
volume and back it up using SQLite's online backup command. For example, after
configuring the app to use `/var/lib/techpulse/techpulse.sqlite3` and mounting
that directory on the host:

```sh
install -d -m 700 /srv/techpulse/backups
umask 077
sqlite3 /srv/techpulse/data/techpulse.sqlite3 \
  ".backup '/srv/techpulse/backups/techpulse-$(date -u +%Y%m%dT%H%M%SZ).sqlite3'"
```

Schedule this command daily using the host's cron or backup service, keep
retention according to available storage, and copy backups off the deployment
host. Validate a backup with:

```sh
sqlite3 /srv/techpulse/backups/techpulse-YYYYMMDDTHHMMSSZ.sqlite3 \
  'PRAGMA integrity_check;'
```

Test restores against a separate database file before relying on the backup.
The path above is a deployment convention for the planned SQLite integration;
the current binary does not read or write it.

## Deployment checklist

- Keep port 3000 bound to loopback and expose it through a secured reverse proxy
  if remote access is needed.
- Review container health and logs after deployment and after updates.
- Rebuild and recreate the container to apply changes to feeds or categories.
- Configure a persistent database volume and automated, verified backups when
  SQLite storage is implemented.
- Record the deployment host, maintenance window, and backup retention policy
  with the operator's infrastructure notes.
