# Peace SQL

### Create a complete Peace database

Just run, then input your postgresql username and password

### Windows
```bash
init_database.bat
```

### Unix
```bash
init_database.sh
```


## Raw

### Export database
```
pg_dump -O --column-inserts -U <your postgresql username> peace > peace.sql
```

### Import database
```
psql -U <your postgresql username> peace < peace.sql
```