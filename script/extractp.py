import re
import sys
import time

with open(sys.argv[1], 'r') as f:
    s = f.read()
    avg = 0
    mm = re.finditer(r'\[.*([0-9]{2,2}):([0-9]{2,2}):([0-9]{2,2})\.([0-9]{2,6})Z INFO  organ::client\] Sent ClientPrifiMessage\.\n\[.*([0-9]{2,2}):([0-9]{2,2}):([0-9]{2,2})\.([0-9]{2,6})Z INFO  organ::client\] Received Server Ok Message', s)
    for m in mm:
        time1 = int(m.group(1)) * 3600 + int(m.group(2)) * 60 + \
            int(m.group(3)) + int(m.group(4)) / 1000000
        time2 = int(m.group(5)) * 3600 + int(m.group(6)) * 60 + \
            int(m.group(7)) + int(m.group(8)) / 1000000
        avg += (time2 - time1)
    avg /= 10
    print("{}".format(round(avg, 6)))
