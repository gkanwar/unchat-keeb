#!/bin/bash

ffmpeg -i "$1" -filter:v "setpts=PTS/30" -an -r 30 "timelapse_$1"
