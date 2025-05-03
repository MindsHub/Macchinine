export ANDROID_HOME="/home/alessio/Android/Sdk"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk"


export PATH=$ANDROID_HOME/tools:$PATH
export PATH=$ANDROID_HOME/platform-tools:$PATH


cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -o app/src/main/jniLibs/  build --release --lib
./gradlew build
./gradlew installDebug
adb shell am start -n com.mindshub.macchinine/.MainActivity