#!/bin/sh
DATABASE_URL=postgres://${TEST_POSTGRES_USER}:${TEST_POSTGRES_PASSWORD}@${TEST_POSTGRES_HOST}/${POSTGRES_DB} && diesel migration run
