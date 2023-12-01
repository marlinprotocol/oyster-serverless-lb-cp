#!/bin/sh

/app/oyster-serverless-lb-cp --config-path /app/config.ini &
echo "Oyster Serverless LB CP is running"

nginx -g "daemon off;" -c /etc/nginx/nginx.conf
