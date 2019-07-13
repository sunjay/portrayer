#!/usr/bin/env bash

# Makes a cube map by repeating a square texture 12 times

if [[ $# -eq 0 ]] ; then
    echo 'Please provide a texture image file as an argument'
    exit 1
fi

OUT="${1%.*}_cubemap.jpg"
echo Writing $OUT
montage -mode concatenate -tile 4x3 $1 $1 $1 $1 $1 $1 $1 $1 $1 $1 $1 $1 $OUT
