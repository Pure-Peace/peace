# read the sql
print('reading and handle...')
with open('peace.sql', 'r', encoding='utf-8') as sql_r:
    content_list = [
        row for row in sql_r.readlines() if not(row.startswith('--'))
    ]

print('writing out...')
with open('peace.sql', 'w', encoding='utf-8') as sql_w:
    sql_w.writelines(content_list)

print('done.')
