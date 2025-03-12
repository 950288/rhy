# Usage

refresh all files
```
rhy -a
```

set config
```
rhy cache-dir /data/rcache
rhy mount-path /remote
rhy remote-path vfs/
```

print the state of x.py
```
rhy -s x.py
```

refresh and print the state of x.py
```
rhy x.py
```

refresh x.py until get the latest update within 15s
```
rhy -t x.py
```
