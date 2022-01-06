import math
import os
import time
import sys

# don't import the local raveberry_visualization folder
del sys.path[0]

import raveberry_visualization

BARS = 256

controller = raveberry_visualization.Controller()
controller.start("Circle", 30, 400, 20)

time_elapsed = 0
last_loop = time.time()
try:
    while True:
        if not controller.is_active():
            break
        current_frame = [
            0.8
            * 0.5
            * (1 + math.sin(4 * time_elapsed))
            * 0.5
            * (1 + math.sin(-4 * time_elapsed + 0.2 * i * 200))
            for i in range(BARS)
        ]
        alarm_factor = -1
        controller.set_parameters(alarm_factor, current_frame)

        now = time.time()
        time_elapsed += now - last_loop
        last_loop = now
        time.sleep(1 / 30)
except KeyboardInterrupt:
    controller.stop()
