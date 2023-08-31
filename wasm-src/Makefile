build:
	wasm-pack build --target web --out-name store 
	sed -i.bak -e 's/"name": "resonance-box-store"/"name": "@resonance-box\/store"/g' pkg/package.json
	rm pkg/package.json.bak

.PHONY: build
