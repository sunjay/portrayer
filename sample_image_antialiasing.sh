#!/usr/bin/env bash

set -e

# Script for generating antialiasing sample image

# Scale up with no antialiasing (preserves image while increasing resolution)
convert antialiasing_1.png -resize 200% +antialias resize_antialiasing_1.png
convert antialiasing_32.png -resize 200% +antialias resize_antialiasing_32.png

# Crop out the corner to get that "enhance" effect for the lower tiles
convert resize_antialiasing_1.png -crop 300x250+0+0 resize_antialiasing_1_crop.png
convert resize_antialiasing_32.png -crop 300x250+0+0 resize_antialiasing_32_crop.png

# Scale up the cropped versions
convert resize_antialiasing_1_crop.png -resize 200% +antialias resize_antialiasing_1_crop2.png
convert resize_antialiasing_32_crop.png -resize 200% +antialias resize_antialiasing_32_crop2.png

# Add labels
composite -background none -pointsize 32 label:"Samples: 1" -geometry +10+10 resize_antialiasing_1.png resize_antialiasing_1_label.png
composite -background none -pointsize 32 label:"Samples: 32" -geometry +10+10 resize_antialiasing_32.png resize_antialiasing_32_label.png

montage -mode concatenate -tile 2x \
  resize_antialiasing_1_label.png resize_antialiasing_1_crop2.png \
  resize_antialiasing_32_label.png resize_antialiasing_32_crop2.png \
  antialiasing.png

rm resize_antialiasing_*.png
