js:
	cargo build --release --target asmjs-unknown-emscripten
	@cp ../target/asmjs-unknown-emscripten/release/js*.js* ../sarc.js

run:
	@python -m SimpleHTTPServer 8080
