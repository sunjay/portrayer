#!/usr/bin/env bash

# Makes a sphere texture by doubleing a square texture along the horizontal axis

if [[ $# -eq 0 ]] ; then
    echo 'Please provide a texture image file as an argument'
    exit 1
fi

OUT="${1%.*}_2.jpg"
echo Writing $OUT
montage -mode concatenate -tile 2x $1 $1 $OUT
