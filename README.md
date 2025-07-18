# rust-mikanos
Rust で作るMikan OS

## 環境構築
[Appleシリコン用環境準備](https://qiita.com/yamoridon/items/4905765cc6e4f320c9b5)を参考にして作った。
- hdiutilが`macOS: Sequoia`から使えなくなってしまったため、｀mtools`を入れておく
- `brew install mtools`
- patchは以下を使う。(cur dir: `mikanos-build`)`patch -p1 < mac.patch`
```diff
--- a/devenv/make_image.sh
+++ b/devenv/make_image.sh
@@ -23,11 +23,24 @@ qemu-img create -f raw $DISK_IMG 200M
 mkfs.fat -n 'MIKAN OS' -s 2 -f 2 -R 32 -F 32 $DISK_IMG
 
 $DEVENV_DIR/mount_image.sh $DISK_IMG $MOUNT_POINT
-sudo mkdir -p $MOUNT_POINT/EFI/BOOT
-sudo cp $EFI_FILE $MOUNT_POINT/EFI/BOOT/BOOTX64.EFI
+if [ `uname` = 'Darwin' ]; then
+    mmd -i disk.img ::/EFI
+    mmd -i disk.img ::/EFI/BOOT
+    mcopy -i disk.img $EFI_FILE ::/EFI/BOOT/BOOTX64.EFI
+else
+    sudo mkdir -p $MOUNT_POINT/EFI/BOOT
+    sudo cp $EFI_FILE $MOUNT_POINT/EFI/BOOT/BOOTX64.EFI
+fi
 if [ "$ANOTHER_FILE" != "" ]
 then
-    sudo cp $ANOTHER_FILE $MOUNT_POINT/
+    if [ `uname` = 'Darwin' ]; then
+        mcopy -i disk.img $ANOTHER_FILE ::/
+    else
+        sudo cp $ANOTHER_FILE $MOUNT_POINT/
+    fi
 fi
 sleep 0.5
-sudo umount $MOUNT_POINT
+if [ `uname` = 'Darwin' ]; then
+    echo "PASS"
+else
+    sudo umount $MOUNT_POINT
+fi
diff --git a/devenv/mount_image.sh b/devenv/mount_image.sh
index ba8233e..aea4d7d 100755
--- a/devenv/mount_image.sh
+++ b/devenv/mount_image.sh
@@ -16,5 +16,9 @@ then
     exit 1
 fi
 
-mkdir -p $MOUNT_POINT
-sudo mount -o loop $DISK_IMG $MOUNT_POINT
+if [ `uname` = 'Darwin' ]; then
+    echo "PASS"
+else
+    mkdir -p $MOUNT_POINT
+    sudo mount -o loop $DISK_IMG $MOUNT_POINT
+fi
```

### rust設定
- target追加: `rustup target add x86_64-unknown-uefi`


# References
- [公式ソースコード](https://github.com/uchan-nos/mikanos)
- [公式サイト](https://zero.osdev.jp/)
- [Appleシリコン用環境準備](https://qiita.com/yamoridon/items/4905765cc6e4f320c9b5)
