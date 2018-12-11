#!/bin/bash
wc -l $(find src/ -name "*.rs") $(find src/ -name "*.lalrpop")

