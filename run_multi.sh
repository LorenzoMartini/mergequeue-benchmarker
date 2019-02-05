#!/usr/bin/env bash
## declare an array variable
declare -a arr=(1000 10000 100000 1000000 10000000 100000000)
declare -a pins=(0 4)
## now loop through the above array
for pin in "${pins[@]}"
do
	for i in "${arr[@]}"
	do
		sleep 10
		echo ./target/release/benchmarker -i 1000000 -f $i -s 0 -r $pin > ./mergequeue_benchmark/"$i"Hz_0-"$pin".log
   		./target/release/benchmarker -i 1000000 -f $i -s 0 -r $pin > ./mergequeue_benchmark/"$i"Hz_0-"$pin".log
	done
done
