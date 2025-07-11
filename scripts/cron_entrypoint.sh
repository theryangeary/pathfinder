#!/bin/bash

# allow running a container which serves the api AND runs cron jobs - this should only be run on a single instance simultaneously

set -m # to make job control work
/app/api-server &
supercronic /app/crontab &
fg %1

