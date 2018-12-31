.PHONY: clean

all:
	bash check-rust-installation.sh
	cargo build --release
	cp target/release/latte-compiler latc_llvm
	echo "./latc_llvm --make-executable \$$@" > latc
	chmod a+x latc

clean:
	rm -Rf latc{,_llvm}
	cargo clean
