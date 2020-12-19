pg_dump -O --column-inserts -U postgres peace > peace.sql
python clear_comments.py
