#!/bin/bash

cd dist/msla_format

rm -r src/ctb src/goo src/nanodlp src/common

cp -r ../../format/ctb_format/src src/ctb
cp -r ../../format/goo_format/src src/goo
cp -r ../../format/nanodlp_format/src src/nanodlp
cp -r ../../common/src src/common

mv src/ctb/lib.rs src/ctb/mod.rs
mv src/goo/lib.rs src/goo/mod.rs
mv src/nanodlp/lib.rs src/nanodlp/mod.rs
mv src/common/lib.rs src/common/mod.rs

grep -RIl '' src/common | xargs sed -i -e 's/use crate/use crate::common/g'
grep -RIl '' src/ctb | xargs sed -i -e 's/use crate/use crate::ctb/g'
grep -RIl '' src/goo | xargs sed -i -e 's/use crate/use crate::goo/g'
grep -RIl '' src/nanodlp | xargs sed -i -e 's/use crate/use crate::nanodlp/g'
grep -RIl '' src/common src/ctb src/goo src/nanodlp | xargs sed -i -e 's/use common/use crate/g'

# cargo publish --dry-run --allow-dirty
