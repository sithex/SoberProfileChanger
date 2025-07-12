This is my first Rust program. I got tired of manually swapping the cookie files for Sober, so I made this.

To use it, go to ~/.var/app/org.vinegarhq.Sober/data/sober/ and copy the cookies file. Rename it to cookies_USERNAME (replacing USERNAME with the appropriate profile name).
The app will automatically detect and list profiles based on the cookies_* files. You don't need to change profile on Roblox.com either since Sober will just use whatever profile is signed in.

To test it, go to the main directory where Sober_logo.png is located and run:

cargo run

To build it, run:

cargo build --release

You may need to manually copy the logo file (Sober_logo.png) into the appropriate output directory, or replace it with your own.

    Note: You must have cargo and rustc installed to build or run the app.


Unfortunately, youll have to remake the cookies when roblox changes your cookie...
