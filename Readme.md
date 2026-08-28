Prerequisites
-------------



- Install cargo via rustup

- Install cargo cross compilation tools:
`cargo install cargo-cross`

Compile
-------

### Bird-Station:
`cross build --target aarch64-unknown-linux-gnu --bin bird-station --release`

### Bird-Display:
`cross build --target arm-unknown-linux-gnueabihf --bin bird-display --release`


- In text and code files, I often use the word "birb" instead of "bird" because it is more cute and funny. Variables are most of the time bird tho to keep it at least somewhat professional.

- Excluded a couple of items in labels.txt that are not birbs, like "Human non-vocal", "Dog", "Fireworks", ....
  The human entries I found instantly during testing because I triggered those a lot. The other ones I found via the regex ^\w\+$ because they all have only one word and the birb names always consists of two words. Maybe there are still some other non-birb entries tho, I didn't check all 6522 lines...

- `scripts/` contains pre-runtime utilities to gather data for displaying birb information. `birb_scraper.py` is scraping Wikipedia for all birb articles that can be found using `model_data/labels.txt`. From these articles, it will download the summary and the first section as well as the first image, if any.  


### Run with:
`docker run -v /run/user/1000/pipewire-0:/tmp/pipewire-0 -e XDG_RUNTIME_DIR=/tmp -t birb`

Offline Encyclopedia
--------------------

This project contains an auto-generated encyclopedia to show basic data for detected birbs. This data has been generated with two python scripts that are included in this repository. Since the encyclopedia is already provided, the user should not need to execute these scripts again.

  - `birb_scraper.py` accesses Wikipedia and looks for all birbs that are found in `model_data/labels.txt`. It initially looks for the English article, but also tries to find the German version and uses this if it is found. From the article, it takes the summary and the first section and puts the text in `description.txt`. It will also look for `.jpg` files in the article and downloads the first one it finds, in the hope that it is a proper visual representation of the birb, and puts it in `image.jpg`.

  - `post_process.py` takes the data that has been scraped by the aforementioned script. The command line parameter `--resize` resizes all images to 800px, keeping the aspect ratio. It then generates an `index.html` for each birb, containing the text from `description.txt` and the image inline-encoded as base64. Finally, it removes the `description.txt` and `image.jpg`, keeping only the `index.html` to display birb information. This shrinks the needed disk space from ~2.4GB to ~521MB.

