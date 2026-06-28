#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PREFIX="$SCRIPT_DIR/out"
SRC_DIR="$SCRIPT_DIR/src"
HOST_OS="$(uname -s)"
HOST_ARCH="$(uname -m)"
JOBS="${JOBS:-$(sysctl -n hw.ncpu 2>/dev/null || nproc 2>/dev/null || echo 8)}"

mkdir -p "$PREFIX/bin" "$PREFIX/lib" "$PREFIX/include" "$SRC_DIR"

export PKG_CONFIG_PATH="$PREFIX/lib/pkgconfig:${PKG_CONFIG_PATH:-}"
export CFLAGS="-I$PREFIX/include ${CFLAGS:-}"
export LDFLAGS="-L$PREFIX/lib ${LDFLAGS:-}"
if [ "$HOST_OS" = "Darwin" ]; then
    export DYLD_LIBRARY_PATH="$PREFIX/lib:${DYLD_LIBRARY_PATH:-}"
else
    export LD_LIBRARY_PATH="$PREFIX/lib:${LD_LIBRARY_PATH:-}"
fi

echo "==> Building APC dependencies"
echo "    prefix: $PREFIX"
echo "    host:   $HOST_OS/$HOST_ARCH"
echo "    jobs: $JOBS"
echo ""

# ─── Step 1: ANGLE ───────────────────────────────────────────────
build_angle() {
    if [ "$HOST_OS" != "Darwin" ]; then
        echo "==> [1/7] Skipping ANGLE (Linux uses native EGL/Mesa)"
        return
    fi

    echo "==> [1/5] Building ANGLE (GLES→Metal)"

    if [ -f "$PREFIX/lib/libEGL.dylib" ] && [ -f "$PREFIX/lib/libGLESv2.dylib" ]; then
        echo "    ANGLE already built, skipping"
        return
    fi

    # gn is required for ANGLE — build from source if not available
    if ! command -v gn &>/dev/null; then
        echo "    Building gn from source..."
        cd "$SRC_DIR"
        if [ ! -d gn ]; then
            git clone https://gn.googlesource.com/gn
        fi
        cd gn
        python3 build/gen.py
        ninja -C out
        cp out/gn "$PREFIX/bin/"
        export PATH="$PREFIX/bin:$PATH"
        cd "$SRC_DIR"
    fi

    cd "$SRC_DIR"
    if [ ! -d angle ]; then
        git clone --depth 1 https://chromium.googlesource.com/angle/angle.git
    fi
    cd angle

    # Bootstrap dependencies
    python3 scripts/bootstrap.py 2>/dev/null || true
    gclient sync --no-history --shallow 2>/dev/null || {
        echo "    Note: gclient sync failed. Trying standalone build..."
        # For standalone builds without depot_tools, we need to handle deps differently
        git submodule update --init --recursive 2>/dev/null || true
    }

    gn gen out/release --args='
        target_cpu="arm64"
        target_os="mac"
        is_debug=false
        is_component_build=true
        angle_enable_metal=true
        angle_enable_vulkan=false
        angle_enable_gl=false
        angle_enable_null=false
        angle_enable_swiftshader=false
        angle_enable_wgpu=false
        angle_has_frame_capture=false
        angle_build_all=false
        use_custom_libcxx=false
        use_lld=false
        use_system_xcode=true
    '

    ninja -C out/release -j"$JOBS" libEGL libGLESv2

    cp out/release/libEGL.dylib "$PREFIX/lib/"
    cp out/release/libGLESv2.dylib "$PREFIX/lib/"

    # Fix install names for rpath-based loading
    install_name_tool -id "@rpath/libEGL.dylib" "$PREFIX/lib/libEGL.dylib"
    install_name_tool -id "@rpath/libGLESv2.dylib" "$PREFIX/lib/libGLESv2.dylib"

    # Copy EGL/GLES headers
    cp -r include/EGL "$PREFIX/include/" 2>/dev/null || true
    cp -r include/GLES2 "$PREFIX/include/" 2>/dev/null || true
    cp -r include/GLES3 "$PREFIX/include/" 2>/dev/null || true
    cp -r include/KHR "$PREFIX/include/" 2>/dev/null || true

    echo "    ANGLE built: libEGL.dylib + libGLESv2.dylib"
}

# ─── Step 2: libepoxy ────────────────────────────────────────────
build_libepoxy() {
    echo "==> [2/5] Building libepoxy (GL dispatch with EGL support)"

    local lib_name="libepoxy.dylib"
    local build_dir="build"
    if [ "$HOST_OS" = "Linux" ]; then
        lib_name="libepoxy.so"
        build_dir="build-linux-$HOST_ARCH"
    fi

    if [ -f "$PREFIX/lib/$lib_name" ]; then
        echo "    libepoxy already built, skipping"
        return
    fi

    cd "$SRC_DIR"
    if [ ! -d libepoxy ]; then
        git clone --depth 1 https://github.com/anholt/libepoxy.git
    fi
    cd libepoxy

    meson setup "$build_dir" \
        --prefix="$PREFIX" \
        --libdir=lib \
        --buildtype=release \
        -Degl=yes \
        -Dx11=false \
        -Dglx=no \
        -Dtests=false

    ninja -C "$build_dir" -j"$JOBS"
    ninja -C "$build_dir" install

    echo "    libepoxy built"
}

# ─── Step 3: virglrenderer ───────────────────────────────────────
build_virglrenderer() {
    echo "==> [3/5] Building virglrenderer (upstream + ANGLE EGL backend)"

    local lib_name="libvirglrenderer.dylib"
    local build_dir="build"
    local drm_opt="-Ddrm=disabled"
    if [ "$HOST_OS" = "Linux" ]; then
        lib_name="libvirglrenderer.so"
        build_dir="build-linux-$HOST_ARCH"
        drm_opt="-Ddrm=enabled"
    fi

    if [ -f "$PREFIX/lib/$lib_name" ]; then
        echo "    virglrenderer already built, skipping"
        return
    fi

    cd "$SRC_DIR"
    if [ ! -d virglrenderer ]; then
        git clone --depth 1 https://gitlab.freedesktop.org/virgl/virglrenderer.git
    fi
    cd virglrenderer

    meson setup "$build_dir" \
        --prefix="$PREFIX" \
        --libdir=lib \
        --buildtype=release \
        -Dvenus=true \
        -Drender-server=false \
        "$drm_opt" \
        -Dtests=false

    ninja -C "$build_dir" -j"$JOBS"
    ninja -C "$build_dir" install

    echo "    virglrenderer built"
}

# ─── Step 4: libkrunfw ───────────────────────────────────────────
build_libkrunfw() {
    echo "==> [4/5] Building libkrunfw (kernel firmware)"

    local lib_name="libkrunfw.dylib"
    if [ "$HOST_OS" = "Linux" ]; then
        lib_name="libkrunfw.so"
    fi

    if [ -f "$PREFIX/lib/$lib_name" ]; then
        echo "    libkrunfw already built, skipping"
        return
    fi

    cd "$SRC_DIR"
    if [ ! -d libkrunfw ]; then
        git clone --depth 1 https://github.com/containers/libkrunfw.git
    fi
    cd libkrunfw

    if [ "$HOST_OS" = "Darwin" ]; then
        make -j"$JOBS"

        cp libkrunfw.dylib "$PREFIX/lib/" 2>/dev/null || \
        cp target/release/libkrunfw.dylib "$PREFIX/lib/" 2>/dev/null || {
            echo "    ERROR: Could not find built libkrunfw.dylib"
            exit 1
        }
        install_name_tool -id "@rpath/libkrunfw.dylib" "$PREFIX/lib/libkrunfw.dylib"
    else
        make PREFIX="$PREFIX" -j"$JOBS" install
        cp -a "$PREFIX/lib64"/libkrunfw.so* "$PREFIX/lib/" 2>/dev/null || true
    fi

    echo "    libkrunfw built"
}

# ─── Step 5: libkrun ─────────────────────────────────────────────
build_libkrun() {
    echo "==> [5/7] Building libkrun (patched for upstream virglrenderer)"

    local lib_name="libkrun.dylib"
    if [ "$HOST_OS" = "Linux" ]; then
        lib_name="libkrun.so"
    fi

    if [ -f "$PREFIX/lib/$lib_name" ]; then
        echo "    libkrun already built, skipping"
        return
    fi

    cd "$SRC_DIR"
    if [ ! -d libkrun ]; then
        git clone --depth 1 https://github.com/containers/libkrun.git
    fi
    cd libkrun

    local libkrun_patch="$SCRIPT_DIR/patches/libkrun/macos-virgl-fence-poll.patch"
    if [ "$HOST_OS" = "Darwin" ] && [ -f "$libkrun_patch" ]; then
        if git apply --reverse --check "$libkrun_patch" >/dev/null 2>&1; then
            echo "    libkrun macOS virgl patch already applied"
        else
            echo "    Applying libkrun macOS virgl patch"
            git apply "$libkrun_patch"
        fi
    fi

    # Build with GPU + input + net support, linking against our custom virglrenderer
    make GPU=1 BLK=1 INPUT=1 NET=1 \
        PREFIX="$PREFIX" \
        LIBRARY_PATH="$PREFIX/lib" \
        C_INCLUDE_PATH="$PREFIX/include" \
        -j"$JOBS" install

    if [ "$HOST_OS" = "Darwin" ]; then
        cp target/release/libkrun.dylib "$PREFIX/lib/" 2>/dev/null || {
            # Try alternative output location
            find . -name "libkrun.dylib" -exec cp {} "$PREFIX/lib/" \;
        }
        install_name_tool -id "@rpath/libkrun.dylib" "$PREFIX/lib/libkrun.dylib"

        # Rewrite any hardcoded dylib paths to use @rpath
        install_name_tool -change /opt/homebrew/opt/virglrenderer/lib/libvirglrenderer.1.dylib \
            @rpath/libvirglrenderer.1.dylib "$PREFIX/lib/libkrun.dylib" 2>/dev/null || true
        install_name_tool -change /opt/homebrew/opt/libepoxy/lib/libepoxy.0.dylib \
            @rpath/libepoxy.0.dylib "$PREFIX/lib/libkrun.dylib" 2>/dev/null || true
    else
        cp -a "$PREFIX/lib64"/libkrun.so* "$PREFIX/lib/" 2>/dev/null || true
    fi

    # Copy headers
    cp include/libkrun.h "$PREFIX/include/" 2>/dev/null || true
    cp include/libkrun_display.h "$PREFIX/include/" 2>/dev/null || true
    cp include/libkrun_input.h "$PREFIX/include/" 2>/dev/null || true

    echo "    libkrun built"
}

# ─── Step 6: glib2 ──────────────────────────────────────────────
build_glib() {
    echo "==> [6/7] Building glib2 (required by libslirp)"

    local lib_name="libglib-2.0.dylib"
    local build_dir="build"
    if [ "$HOST_OS" = "Linux" ]; then
        lib_name="libglib-2.0.so"
        build_dir="build-linux-$HOST_ARCH"
    fi

    if [ -f "$PREFIX/lib/$lib_name" ]; then
        echo "    glib2 already built, skipping"
        return
    fi

    cd "$SRC_DIR"
    if [ ! -d glib ]; then
        git clone --depth 1 --branch 2.84.1 https://github.com/GNOME/glib.git
    fi
    cd glib

    meson setup "$build_dir" \
        --prefix="$PREFIX" \
        --libdir=lib \
        --buildtype=release \
        -Dtests=false \
        -Ddtrace=disabled \
        -Dintrospection=disabled \
        -Dnls=disabled \
        -Dman-pages=disabled

    ninja -C "$build_dir" -j"$JOBS"
    ninja -C "$build_dir" install

    echo "    glib2 built"
}

# ─── Step 7: libslirp ───────────────────────────────────────────
build_libslirp() {
    echo "==> [7/7] Building libslirp (userspace TCP/IP stack)"

    local lib_name="libslirp.dylib"
    local build_dir="build"
    if [ "$HOST_OS" = "Linux" ]; then
        lib_name="libslirp.so"
        build_dir="build-linux-$HOST_ARCH"
    fi

    if [ -f "$PREFIX/lib/$lib_name" ]; then
        echo "    libslirp already built, skipping"
        return
    fi

    cd "$SRC_DIR"
    if [ ! -d libslirp ]; then
        git clone --depth 1 https://gitlab.freedesktop.org/slirp/libslirp.git
    fi
    cd libslirp

    meson setup "$build_dir" \
        --prefix="$PREFIX" \
        --libdir=lib \
        --buildtype=release

    ninja -C "$build_dir" -j"$JOBS"
    ninja -C "$build_dir" install

    echo "    libslirp built"
}

# ─── Run ─────────────────────────────────────────────────────────

build_angle
build_libepoxy
build_virglrenderer
build_libkrunfw
build_libkrun
build_glib
build_libslirp

echo ""
echo "==> All dependencies built successfully"
echo "    Libraries:"
if [ "$HOST_OS" = "Darwin" ]; then
    ls -lh "$PREFIX/lib/"*.dylib
else
    ls -lh "$PREFIX/lib/"*.so*
fi
echo ""
echo "    Set in build.rs:"
echo "    cargo:rustc-link-search=$PREFIX/lib"
if [ "$HOST_OS" = "Darwin" ]; then
    echo "    cargo:rustc-link-arg=-Wl,-rpath,@executable_path/lib"
else
    echo "    cargo:rustc-link-arg=-Wl,-rpath,$PREFIX/lib"
fi
