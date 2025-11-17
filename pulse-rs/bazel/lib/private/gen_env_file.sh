#!/bin/bash

FILE=$1

touch $FILE

# LIBRARY_PATH, if set
if [[ ! -z "${LIBRARY_PATH}" ]]; then
    echo "LIBRARY_PATH=$LIBRARY_PATH" >> $FILE
fi
