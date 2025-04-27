If the last line is empty (e.g. trailing newline),
e.g. foo\n
iterator should have units_total = 2 but if bytes_left = 0 then yields empty slice

otherwise,
it can yield slice as normal
