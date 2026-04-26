#!/opt/python-venv/bin/python3

import os

import wikipediaapi
import requests
import argparse
from PIL import Image

from config import encyclopedia_dir
from config import labels_file


def create_dirs():
    """
    Create the directory hierarchy for our encyclopedia.
    """

    with open(labels_file) as file:
        # first we create the root directory for our encyclopedia
        try:
            os.mkdir(encyclopedia_dir)
            print(f"Directory '{encyclopedia_dir}' created successfully.")
        except FileExistsError:
            pass
        except PermissionError:
            print(f"Permission denied: Unable to create '{encyclopedia_dir}'.")
        except Exception as e:
            print(f"An error occurred: {e}")

        # then, for each birb, we create a sub directory
        for line in file:
            directory_name = encyclopedia_dir + "/" + line.rstrip()
            try:
                os.mkdir(directory_name)
                print(f"Directory '{directory_name}' created successfully.")
            except FileExistsError:
                pass
            except PermissionError:
                print(f"Permission denied: Unable to create '{
                      directory_name}'.")
            except Exception as e:
                print(f"An error occurred: {e}")


def get_page_in_language(page, lang):
    """
    For a given article in the English Wikipedia,
    get the article in another language, if there is one.
    """
    langlinks = page.langlinks
    if lang in langlinks:
        new_page = langlinks[lang]
        return new_page
    else:
        return None


def get_first_valid_section(page):
    """
        Get the text of the first section of a birb article.

        This looks a bit unprofessional but we can't really use common
        tree-traversal techniques, because sometimes, the text is under
        the first section, even though we have more subsections, sometimes
        we have no text at the first section but instantly the subsection
        of the section, sometimes we don't have any sections at all.
        This code will only work with a section level=2, i.e.

    >        |---------|
    >        | Section |
    >        |---------|
    >
    >        Texttexttexttexttexttext
    >        texttext
    >
    >        |------------|
    >        | Subsection |
    >        |------------|
    >         ...
    >
        or
    >
    >        |---------|
    >        | Section |
    >        |---------|
    >        |------------|
    >        | Subsection |
    >        |------------|
    >
    >        Texttexttexttexttexttext
    >        texttext
    >        ...


        does work, but no more levels, which we also don't expect
        from wikipedia standards.
    """
    sections = page.sections
    if len(sections) > 0 and len(sections[0].text) > 0:
        return sections[0].text

    if len(sections) > 0:
        subsections = sections[0].sections
        if len(subsections) > 0:
            return subsections[0].text

    return ""


def scrape(bird_name, no_images: False):
    print("==== Scraping:", bird_name, "====")
    user_agent = (
        "BirbDetection (One-Time Scrape for Offline Mode)"
        "(Marian Cichy <mail@majoran.net>)"
    )
    wiki = wikipediaapi.Wikipedia(
        user_agent=user_agent,
        language="en",
        extract_format=wikipediaapi.ExtractFormat.WIKI,
    )

    headers = {"User-Agent": user_agent}

    # set the file names for our birb information.
    bird_dir = encyclopedia_dir + "/" + bird_name + "/"
    description_file_name = bird_dir + "description.txt"
    image_file_name = bird_dir + "image.jpg"

    # skip texts already scraped to not overload wiki servers each test
    if os.path.isfile(description_file_name):
        return
    # get the English Wikipedia article. Also look up, if there is a
    # corresponding German article. We'll use the German if we have one,
    # otherwise we use the English one.
    en_page = wiki.page(bird_name)
    de_page = get_page_in_language(en_page, "de")

    if de_page is not None:
        page = de_page
    else:
        page = en_page

    # sanity check if we have an article at all. If not,
    # we don't continue to scrape this birb any further.
    if len(page.text) < 1:
        return

    # we want the first summary of the article...
    summary = page.summary
    # ...and the first section, which is usually the description.

    first_section = get_first_valid_section(page)
    text = summary + "\n\n" + first_section

    # finally, write description.txt of this birb.
    f = open(description_file_name, "w")
    f.write(text)
    f.close()

    # Now, we have the description text, but we also want an image of our birb.
    # Wikipedia-API gives us a way to get all images of an article. We iterate
    # through the images and take the first one that is an .jpg (Otherwise,
    # we'll get some .svg's that are just Wikipedia logos). We then
    # try to download this image. Because Wikipedia has some DDoS guards,
    # we might get an 429 (Too Many Requests). We try again until we have
    # an 200, but no more than 10 times per birb.
    # NOTE: Since I am using a proper User-Agent with name, mail-address,
    # NOTE: and intention, I don't get any 429 anymore. Net etiquette still exists!

    # skip images already scraped to not overload wiki servers each test
    if not no_images:
        if not os.path.isfile(image_file_name):
            for title, img in page.images.items():
                if ".jpg" in img.url:
                    status_code = 0
                    request_count = 0
                    while status_code != 200 and request_count < 10:
                        img_data = requests.get(
                            img.url, headers=headers, stream=True)

                        status_code = img_data.status_code
                        if status_code != 200:
                            print("[GET ERROR]", status_code, "Retrying...")
                            request_count += 1

                    if status_code == 200:
                        img = Image.open(img_data.raw)
                        img.save(bird_dir + "image.jpg")

                    break  # break after first .jpg found


def main():
    parser = argparse.ArgumentParser(
        description="Post-process bird encyclopedia data.")
    parser.add_argument(
        "--bird", default=None, help="Scrape only a single birb instead of all birbs."
    )
    parser.add_argument(
        "--no-images",
        action="store_true",
        default=False,
        help="Disable scraping images.",
    )

    args = parser.parse_args()
    # PIL is unhappy with the size of some wiki images, so we have to increase.
    Image.MAX_IMAGE_PIXELS = 933120000

    create_dirs()

    if args.bird is not None:
        scrape(args.bird, args.no_images)
    else:
        with open(labels_file) as file:
            for birb in file:
                birb = birb.rstrip()
                scrape(birb, args.no_images)


if __name__ == "__main__":
    main()
