FROM ghcr.io/cross-rs/x86_64-pc-windows-gnu:main

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        cmake \
        clang \
        libclang-dev \
        mingw-w64-tools \
        mingw-w64-x86-64-dev \
        g++-mingw-w64-x86-64 \
    && rm -rf /var/lib/apt/lists/* \
    && ln -sf windows.h /usr/x86_64-w64-mingw32/include/Windows.h \
    && ln -sf winsock2.h /usr/x86_64-w64-mingw32/include/WinSock2.h \
    && ln -sf ws2tcpip.h /usr/x86_64-w64-mingw32/include/WS2tcpip.h \
    && GCCDIR=$(ls -d /usr/lib/gcc/x86_64-w64-mingw32/*-posix 2>/dev/null | head -1) \
    && ln -sf "$GCCDIR/libstdc++.a" /usr/x86_64-w64-mingw32/lib/libstdc++.a \
    && ln -sf "$GCCDIR/libgcc.a" /usr/x86_64-w64-mingw32/lib/libgcc.a \
    && rm -f /usr/x86_64-w64-mingw32/lib/libstdc++.dll.a \
    && rm -f /usr/lib/gcc/x86_64-w64-mingw32/*/libstdc++.dll.a
