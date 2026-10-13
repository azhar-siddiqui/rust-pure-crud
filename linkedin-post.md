# LinkedIn post — Rust Control

Paste the post below and attach the screen recording. The first two lines are the hook people see before "see more".

## Post

I built a desktop records app in Rust with vibe coding, then checked the binary size and how it runs before I posted it.

The app is called Rust Control. It is a local workspace for creating, searching, editing, and deleting records. Everything stays on the machine as JSON. There is no server.

The stack is small: Rust, eframe and egui for the UI, serde for the records, and the directories crate for the save path. The window has an overview, a records table, light and dark theme, form validation, delete confirmation, and a toast when a save succeeds or fails.

Vibe coding was the build loop. I described the screen, the rules, and the save behavior in plain language, reviewed the Rust that came back, and kept going until the app matched that description. The source is about 2,900 lines across six files.

Two checks are in the video:

File size. The debug build on my Mac is 20 MB. I also show the release binary after `cargo build --release`.

Performance. Cold start, scrolling the record list, and memory use while the window is open.

Vibe coding gets a working UI up fast. The size check and the performance check are what make it something I am willing to show.

#Rust #VibeCoding #egui #DesktopApp #BuildInPublic #LearningInPublic

## Video (60–90 seconds)

Record in this order so the caption matches the video.

1. Open Rust Control. Show Overview, then Records. Create one record, search it, edit it, delete it. Flip light and dark.
2. Terminal, file size. Run these and leave the numbers on screen:

```bash
find src -name '*.rs' | xargs wc -l
ls -lh target/debug/rust-pure-crud
cargo build --release
ls -lh target/release/rust-pure-crud
```

3. Performance. Start the release app, then in another terminal:

```bash
ps -o rss,pid,comm -p "$(pgrep -x rust-pure-crud)"
```

RSS is memory in kilobytes. Scroll the records list while that number is visible. Quit the app once.

After you record, if the release binary is not 20 MB, add one line under the file-size sentence: "Release build: X MB."

## Upload settings

LinkedIn accepts MP4, H.264 video, AAC audio, 1920×1080, 30 fps, 16:9. Keep the file under 200 MB so it finishes processing. LinkedIn allows up to 5 GB and 10 minutes. A vertical 9:16 cut of the same demo also works if you post from your phone.
