build:
	npx tsc

clean:
	rm -rf dist tsconfig.tsbuildinfo

watch:
	npx nodemon ./node_modules/.bin/tsc --watch src -e ts

generate: build
	node ./dist/tool/generate-ast.js

fresh: clean build
