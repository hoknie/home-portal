set shell := ["bash", "-c"]

CARGO := env('CARGO', 'cargo')
PNPM := env('PNPM', 'pnpm')
WEB := 'web'

[private]
default:
    @{{ just_executable() }} --list

import 'env/justice/quality.just'
import 'env/justice/web.just'
import 'env/justice/build.just'
import 'env/justice/run.just'
import 'env/justice/package.just'
import 'env/justice/release.just'
