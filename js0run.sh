set -x
./bin/js0c _data/js0/$1.js _data/out/$1.qd0
./bin/qd0vm _data/out/$1.qd0
./bin/qd0c _data/out/$1.qd0
clang -Wno-override-module -S _data/out/$1.ll -o _data/out/$1.s
clang -Wno-override-module _data/out/$1.ll bin/qd0lib.a -o _data/out/$1
./_data/out/$1
