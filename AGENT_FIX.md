# Fix failing GitHub Actions job for `md-renderer`

## Goal
Fix the failing GitHub Actions job associated with this run:
- URL: https://github.com/pTSern/md-renderer/actions/runs/38677807185/job/116235288473

## Observed failure
The job log ends with:

```text
cp: cannot stat 'android/app/build/outputs/apk/debug/*.apk': No such file or directory
##[error]Process completed with exit code 1.
```

This indicates the workflow is trying to copy an APK file from `android/app/build/outputs/apk/debug/` before any Android build produced that artifact, or it is using the wrong path.

## Likely root cause
The workflow's artifact step assumes an Android APK exists at a specific location, but the project either:
- never builds the Android app in that job,
- builds to a different output path,
- or runs on a runner/configuration that does not produce the APK.

## What to inspect
1. The exact workflow file that triggered the failed job.
2. Any Android build step before the copy step.
3. The Android project layout and whether `android/app/build/outputs/apk/debug/*.apk` is the correct destination.
4. Whether the job is building `debug` vs `release` APKs, and whether the artifact path matches the actual Gradle output.
5. If this repo doesn't actually contain an Android app, confirm the workflow is not referencing stale or incorrect artifact paths.

## Recommended fix
- Add or correct the Android Gradle build step before copying artifacts.
- Ensure the build output path is valid, e.g.:
  - `android/app/build/outputs/apk/debug/*.apk`
  - or `android/app/build/outputs/apk/release/*.apk`
- Before copying, check that the files exist and fail with a clear message if not.
- Prefer `actions/upload-artifact` over raw `cp` for workflow artifacts.

Example pattern:

```yaml
- name: Build Android debug APK
  working-directory: android
  run: ./gradlew assembleDebug --no-daemon --stacktrace --info

- name: Upload APK artifact
  uses: actions/upload-artifact@v4
  with:
    name: app-debug-apk
    path: android/app/build/outputs/apk/debug/*.apk
```

Or if release APK is intended:

```yaml
- name: Build Android release APK
  working-directory: android
  run: ./gradlew assembleRelease --no-daemon --stacktrace --info

- name: Upload APK artifact
  uses: actions/upload-artifact@v4
  with:
    name: app-release-apk
    path: android/app/build/outputs/apk/release/*.apk
```

## Acceptance criteria
- The workflow no longer crashes on `cp: cannot stat .../*.apk`.
- The APK is built before packaging.
- The artifact path matches the actual Gradle output.
- The CI job passes successfully.

## Notes
The current workflow file in the repo should be checked carefully; the failure suggests a stale/inconsistent artifact path or missing Android build step. Verify the exact workflow file and commit that triggered the run before patching.
