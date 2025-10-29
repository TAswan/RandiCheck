# benchmarking with hyperfine

# first quickcheck in haskell
hyperfine --warmup 3 --runs 5 --show-output 'cabal run'
