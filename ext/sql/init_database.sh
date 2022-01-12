#! /bin/bash

read -p "input your postgresql username: " user
psql -U $user < batch.sql
read -n1 -p "Press any key to continue..."
exit