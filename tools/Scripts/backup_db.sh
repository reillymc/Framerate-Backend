#!/bin/sh
DATE=$(date +%Y%m%d-%H%M%S)
docker compose exec -it framerate-database pg_dumpall -U postgres --inserts --data-only | gzip -9c > db-$DATE.sql.gz 
