all: haskell rust

haskell:
	cd src && cabal build

rust:
	cargo build

clean:
	cargo clean
	rm output.essence 
	rm -rf conjure-output
	rm output.solution
	rm validation.*
	