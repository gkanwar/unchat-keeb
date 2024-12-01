#!/bin/bash
ffmpeg -i timelapse_cut.mkv -ss 0 -t 10 -vf "fps=10,scale=512:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse" -loop 0 timelapse_2023-12-22.gif
