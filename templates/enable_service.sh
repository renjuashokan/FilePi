#!/bin/sh
SERVICE="filepi.service"

if systemctl --quiet is-active $SERVICE
    then
        systemctl stop $SERVICE
        systemctl disable $SERVICE
fi

cp -f filepi.service /etc/systemd/system/

systemctl daemon-reload
systemctl enable filepi.service
systemctl start filepi.service

