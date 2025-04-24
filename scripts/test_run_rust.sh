## desktop02
make -j 1 CONFIG_PATH=/home/ubuntu/git_repos/demikernel/scripts/config/config.yaml PROFILER=yes DEBUG=no test-system-rust LIBOS=catnap TEST=tcp-echo ARGS="--peer server --address 10.0.2.102:12345 --nthreads 8"
## desktop01
make -j 1 CONFIG_PATH=/home/ubuntu/git_repos/demikernel/scripts/config/config.yaml PROFILER=yes DEBUG=no test-system-rust LIBOS=catnap TEST=tcp-echo ARGS="--peer client --address 10.0.2.102:12345 --nclients 8 --nrequests 1024 --bufsize 102400 --run-mode concurrent"
