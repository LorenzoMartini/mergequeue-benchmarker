# mergequeue-benchmarker
Simple benchmark for MergeQueues

Please enter the clock frequency of the machine you are running this on as `-c` parameter, in order to have a correct execution.
Output for latency is in clock cycles.

## Instructions
We have 3 different executables, to measure MergeQueue latency under contention, to measure lock acquisition without contention and an additional benchmark to know what's the cost of our measuring infrastructure.
We measure everything in clock cycles with a cycle counter.

To run any, compile it normally and then execute. For the MergeQueues ones you can set up custom args like sender frequency in Hz (`-f`), sender pinning (`-s`), recv pinning (`-r`), number of interations (`-i`). It is important also to insert the clock frequency of the machine you run experiments on (`-c`) to correcly interpret the sender frequency.
