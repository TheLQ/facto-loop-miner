set -x

rm out.mp4
ffmpeg -framerate 2 -i raw-output-stream.dat -loop 0 -c:v libx265 out.mp4
rm out.webp
ffmpeg -framerate 2 -i raw-output-stream.dat -loop 0 out.webp