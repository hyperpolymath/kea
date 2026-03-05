FROM gcr.io/oss-fuzz-base/base-builder-rust
RUN apt-get update && apt-get install -y make autoconf automake libtool
COPY . $SRC/kea-mandible
WORKDIR $SRC/kea-mandible
COPY .clusterfuzzlite/build.sh $SRC/
