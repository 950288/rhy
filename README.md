# Usage

refresh all files
```
rhy -a
```

set config
```
rhy -config mount_path /remote/
```

print the state of x.py
```
rhy -s x.py
```

refresh and print the state of x.py
```
rhy -r x.py
```

refresh x.py until get the latest update within 20s
```
rhy -r x.py -t 20s
```
