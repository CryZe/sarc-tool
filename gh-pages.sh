#!/bin/bash

git branch -D gh-pages
git checkout -b gh-pages
cp target/asmjs-unknown-emscripten/release/js*.js* sarc.js
git add -f sarc.js
git commit -m "gh pages"
git push -f origin head
git checkout -
cp target/asmjs-unknown-emscripten/release/js*.js* sarc.js
