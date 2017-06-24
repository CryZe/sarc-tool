#!/bin/bash

git branch -D gh-pages
git checkout -b gh-pages
git add -f sarc.js
git commit -m "gh pages"
git push -f origin head
git checkout -
