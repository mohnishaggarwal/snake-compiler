#!/bin/bash

cp examples/egg/interesting.egg examples/
zip -r submission.zip Cargo.toml src/ examples/ tests/ runtime/ docs/*.pdf
rm examples/interesting.egg
