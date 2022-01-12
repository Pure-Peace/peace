set /p user=input your postgresql username: 
psql -U %user% < batch.sql
pause