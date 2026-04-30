import os
import base64
import argparse
from pathlib import Path
from PIL import Image
from config import encyclopedia_dir, root_dir


def resize_image(image_path, size=(600, 600)):
    if not image_path.exists():
        return

    try:
        with Image.open(image_path) as img:
            # Resample Image in-place to 600px
            img.thumbnail(size, Image.Resampling.LANCZOS)
            img.save(image_path, "JPEG")
            print(f"Resized image of {image_path.parent.name} to {img.size}")
    except Exception as e:
        print(f"Error resizing {image_path}: {e}")


def get_translations(labels_file):
    translations = {}
    if os.path.exists(labels_file):
        with open(labels_file, "r", encoding="utf-8") as f:
            for line in f:
                parts = line.strip().split(":", 1)
                if len(parts) == 2:
                    translations[parts[0].strip()] = parts[1].strip()
    return translations


def generate_description(bird_name, translation, description):
    desc = f"{bird_name}\n{translation}\n{description}"
    return desc
        


def main():
    parser = argparse.ArgumentParser(
        description="Post-process bird encyclopedia data.")
    parser.add_argument("--resize", action="store_true",
                        help="Resize images to 800px.")
    args = parser.parse_args()

    base_dir = Path(encyclopedia_dir)
    labels_de_file = Path(root_dir) / "scripts" / "labels_de.txt"
    translations = get_translations(labels_de_file)

    if not base_dir.exists():
        print(f"Directory {base_dir} does not exist.")
        return

    # -- Resize images first if requested -- #
    if args.resize:
        for bird_dir in base_dir.iterdir():
            image_file = bird_dir / "image.jpg"
            resize_image(image_file)

    # -- Generate description text -- #
    for bird_dir in base_dir.iterdir():
        bird_name = bird_dir.name
        description_file = bird_dir / "description.txt"
        output_file = bird_dir / "desc.txt"

        translation = translations.get(bird_name, bird_name)

        description = "No description available."
        if description_file.exists():
            with open(description_file, "r", encoding="utf-8") as f:
                description = f.read()
                # our scraper took the summary and the first section
                # for each birb. In small articles, the first section
                # is the last that only contains references. In these
                # cases, we want to remove them.
                # TODO: It'd be cleaner if the scraper had already paid
                # attention to this, but I didn't want to scrape
                # everything again...
                idx = description.rfind("== References")
                if idx != -1:
                    description = description[:idx]

        desc = generate_description(
            bird_name, translation, description)

        try:
            with open(output_file, "w", encoding="utf-8") as f:
                f.write(desc)
        except:
            pass

        print(f"Generated {output_file}")

    # -- Remove old files -- #
        #    for bird_dir in base_dir.iterdir():
        #        description_file = bird_dir / "description.txt"
        #        image_file = bird_dir / "image.jpg"
        #        try:
        #            os.remove(description_file)
        #        except Exception:
        #            pass
        #        try:
        #            os.remove(image_file)
        #        except Exception:
        #            pass


if __name__ == "__main__":
    main()
