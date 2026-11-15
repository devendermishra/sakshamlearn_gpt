"""Convert the six Saksham worksheet packs into the app's offline catalog.

Run with the folder containing the original Word files, then the output JSON path.
OOXML paragraph/table positions identify specific answer adaptations in the originals.
"""
import argparse
import json
import operator
import re
import xml.etree.ElementTree as ET
from pathlib import Path
from zipfile import ZipFile

W = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"
PACKS = [
    ("en-activities", "en", "English Practice Activities", "Claude_english_worksheets"),
    ("en-chapters", "en", "English Chapter Worksheets", "English_Worksheets"),
    ("hi-activities", "hi", "हिंदी अभ्यास", "Claude_hindi_worksheets"),
    ("hi-chapters", "hi", "अध्याय के कार्यपत्रक", "Hindi_Worksheets"),
    ("math-activities", "math", "Mathematics Activity Sheets", "Mathematics_Activity_Sheets"),
    ("math-chapters", "math", "Mathematics Chapter Worksheets", "Mathematices_Worksheets"),
]
OPS = {"+": operator.add, "-": operator.sub, "×": operator.mul, "x": operator.mul, "÷": operator.floordiv}


def paragraph(element):
    parts = []
    for node in element.iter():
        if node.tag == W + "t":
            parts.append(node.text or "")
        elif node.tag in (W + "br", W + "cr"):
            parts.append("\n")
        elif node.tag == W + "tab":
            parts.append("    ")
    return "".join(parts).strip()


def paragraphs(cell):
    return [text for p in cell.iter(W + "p") if (text := paragraph(p))]


def table(element):
    return [["\n".join(paragraphs(cell)) for cell in row.findall(W + "tc")]
            for row in element.findall(W + "tr")]


def outline(path):
    with ZipFile(path) as doc:
        body = ET.fromstring(doc.read("word/document.xml")).find(W + "body")
    return list(body)


def clean(text):
    text = re.sub(r"_{2,}", "___", text)
    text = text.replace("🌟 ", "").strip()
    return re.sub(r"[ \t]+", " ", text)


def useful(text):
    return bool(re.search(r"[\w○△★♦◯🔴|~–]", text)) and not re.fullmatch(r"[\d_.: \n]+", text)


def strip_number(text):
    return re.sub(r"^\s*\d{1,2}[.)]\s+", "", text).strip()


def sheet(pack, grade, number, title, adult=""):
    slug, subject, label, _ = pack
    return dict(id=f"{slug}-{number:02}", subject=subject, pack=label,
                grade=grade, title=clean(title), adult=clean(adult), tasks=[])


def add(result, kind, prompt, context="", hint="", key="", **fields):
    prompt = clean(prompt)
    if not prompt:
        return
    fields.update(kind=kind, prompt=prompt)
    for name, value in (("context", context), ("hint", hint), ("key", key)):
        if value.strip() and clean(value) != prompt:
            fields[name] = clean(value)
    result["tasks"].append(fields)


def choice(result, prompt, options, answer, context="", hint=""):
    options = list(dict.fromkeys(str(option).strip() for option in options if str(option).strip()))
    answer = str(answer)
    if answer not in options or len(options) < 2:
        raise ValueError(f"Invalid options {options!r} / {answer!r} for {prompt!r}")
    # Rotate the options using the task's stable position.
    shift = len(result["tasks"]) % len(options)
    options = options[shift:] + options[:shift]
    add(result, "choice", prompt, context, hint, answer, options=options, correct=options.index(answer))


def number(result, prompt, answer, context="", hint=""):
    answer = str(answer)
    add(result, "number", prompt, context, hint, answer, answer=answer)


def order(result, prompt, sequence, context="", shown=None):
    options = shown if shown else sequence[1::2] + sequence[::2]
    if len(set(sequence)) != len(sequence):
        raise ValueError(f"Duplicate item in sequence {sequence!r}")
    add(result, "order", prompt, context, key=" → ".join(sequence),
        options=options, order=[options.index(item) for item in sequence])


def matching(result, pairs, prompt, choices=None):
    options = list(dict.fromkeys(choices or [right for _, right in pairs]))
    for position, (left, right) in enumerate(pairs):
        if len(options) > 5:
            distractors = [item for item in options if item != right]
            start = position % len(distractors)
            options_for_item = [right] + (distractors[start:] + distractors[:start])[:3]
        else:
            options_for_item = options
        choice(result, left, options_for_item, right, prompt)


def truth(result, claims, values, context="", hindi=False):
    options = ("सही", "गलत") if hindi else ("True", "False")
    for claim, value in zip(claims, values, strict=True):
        choice(result, strip_number(claim), options, options[0 if value else 1], context)


def arithmetic(result, raw, context=""):
    text = clean(raw)
    expression = re.fullmatch(r"(?:₹\s*)?(\d+(?:\s*[+\-x×÷]\s*₹?\s*\d+)+)\s*(?:=\s*(?:___|\?)?)?", text)
    if expression:
        parts = re.split(r"\s*([+\-x×÷])\s*", expression[1].replace("₹", ""))
        value = int(parts[0])
        for op, right in zip(parts[1::2], parts[2::2], strict=True):
            value = OPS[op](value, int(right))
        if value >= 0:
            number(result, expression[1] + " = ___", value, context)
            return True
    return False


def simple_math(result, prompt, context=""):
    text = clean(prompt)
    if arithmetic(result, text, context):
        return True
    if found := re.search(r"(\d+) and ___ make (\d+)", text):
        number(result, text, int(found[2]) - int(found[1]), context, "Count on from the first number.")
    elif found := re.search(r"(\d+) tens? and (\d+) (?:ones?|one)\b", text, re.I):
        number(result, text, 10 * int(found[1]) + int(found[2]), context)
    elif found := re.search(r"^(\d+) tens = ___", text):
        number(result, text, 10 * int(found[1]), context)
    elif found := re.search(r"^(?:What|Which)(?: number)? comes (before|after) (\d+)\?", text):
        number(result, text, int(found[2]) + (1 if found[1] == "after" else -1), context)
    elif found := re.search(r"(?:bigger|greater|smaller)(?: number)?: (\d+) or (\d+)", text):
        values = (int(found[1]), int(found[2]))
        choice(result, text, values, (min if "smaller" in text else max)(values), context)
    elif found := re.search(r"(\d+) ___ (\d+)$", text):
        left, right = int(found[1]), int(found[2])
        if ">" in text:
            choice(result, f"{left} ___ {right}", ["<", "=", ">"], compare(left, right), "Compare the numbers.")
        elif right - left == 2:
            number(result, text, left + 1, context)
        else:
            return False
    elif found := re.fullmatch(r"___ (\d+) ___", text):
        value = int(found[1])
        number(result, f"{value}  ·  What comes before?", value - 1)
        number(result, f"{value}  ·  What comes after?", value + 1)
    elif found := re.fullmatch(r"(\d+),\s*___,\s*(\d+)", text):
        first, last = int(found[1]), int(found[2])
        if last - first == 2:
            number(result, text, first + 1, context)
        else:
            return False
    elif found := re.fullmatch(r"((?:\d+,\s*){3,})(___(?:,\s*___)*)", text):
        known = [int(part) for part in re.findall(r"\d+", found[1])]
        step = known[1] - known[0]
        if step <= 0 or not all(right - left == step for left, right in zip(known, known[1:])):
            return False
        answer = ",".join(str(known[-1] + step * k) for k in range(1, found[2].count("___") + 1))
        number(result, text, answer, "Fill the blanks. Separate two answers with a comma." if "," in answer else context)
    elif found := re.search(r"(?:total =|how many|count).*?(\d+) groups of (\d+)(?: = ___)?", text, re.I):
        number(result, text, int(found[1]) * int(found[2]), context)
    elif found := re.search(r"Count (\d+) counters and write", text, re.I):
        value = int(found[1])
        number(result, "How many counters?", value, "● " * value)
    elif found := re.fullmatch(r"Order: ((?:\d+, )+\d+)", text):
        sequence = sorted(found[1].split(", "), key=int)
        order(result, "Tap from smallest to largest.", sequence)
    else:
        return False
    return True


def compare(first, second):
    return "<" if first < second else ">" if first > second else "="


def practice(result, text, context="", guidance=""):
    text = clean(strip_number(text))
    if not text:
        return
    if result["subject"] == "math" and simple_math(result, text, context):
        return
    lower = text.lower()
    if found := re.match(r"trace (?:and (?:copy|write))?:\s*(.+)", text, re.I):
        models = [item.strip() for item in re.split(r"(?<=\.)\s+|,\s*", found[1]) if item.strip()]
        if all(len(item) <= 36 for item in models):
            for model in models:
                add(result, "trace", model, context, model=model)
            return
    if any(word in lower for word in ("draw", "colour ", "color ", "बनाओ", "चित्र बन", "paint")):
        add(result, "draw", text, context, guidance)
    elif (any(word in lower for word in ("write", "trace", "complete", "copy", "fill", "label", "join with", "___", "लिख", "पूरा", "उत्तर"))
          and not lower.startswith(("say or write", "listen", "read aloud"))):
        add(result, "write", text, context, guidance)
    else:
        add(result, "adult", text, context, guidance)


def standard_english(pack, body):
    collected, current = [], None
    grade = 1
    for index, element in enumerate(body):
        if element.tag == W + "p":
            title = paragraph(element)
            style = element.find(f"{W}pPr/{W}pStyle")
            if index > 11 and index < 568 and style is not None and style.get(W + "val") == "Heading1":
                current = sheet(pack, grade, len(collected) + 1, title,
                    "Give one prompt at a time. Model once. Accept pointing, speaking, or writing. "
                    "Check whether the learner can repeat two examples after a break.")
                collected.append(current)
        elif element.tag == W + "tbl":
            rows = table(element)
            if rows == [["Class 2"]]:
                grade = 2
            if current and rows and rows[0][:2] == ["Step", "Worksheet task"]:
                for _, text, *_ in rows[1:]:
                    english_goal_task(current, text)
    return collected


def english_goal_task(result, text):
    if text.startswith("Choose yes/no:"):
        truth(result, ["Can hands clap?", "Can eyes smell?"], [True, False])
    elif text.startswith("Match word to picture: hand"):
        matching(result, [(word, icon) for word, icon in zip(
            ["hand", "leg", "eye", "ear", "nose", "mouth"], ["✋", "🦵", "👁", "👂", "👃", "👄"], strict=True)],
            "Tap the picture that matches the word.")
    elif text.startswith("Draw line: cow-moo"):
        matching(result, [("Cow", "moo"), ("Duck", "quack"), ("Hen", "cluck")], "Tap the animal’s sound.")
    elif text.startswith("Number 1-2-3-4 picture cards") or text.startswith("Cut and paste story order"):
        order(result, "Tap what happened first, next, and last.", [
            "The cap-seller rested.", "The monkeys took the caps.",
            "The cap-seller threw his cap down.", "The monkeys copied him."])
    elif text.startswith("Sort fruit / vegetable"):
        matching(result, [(word, category) for category, words in (
            ("Fruit", ["apple", "banana", "mango"]), ("Vegetable", ["carrot", "pea", "potato"]))
            for word in words], "Tap fruit or vegetable.")
    elif text.startswith("Complete sentence: I say"):
        choice(result, "I say ___ when I get help.", ["thank you", "good night", "goodbye"], "thank you")
    else:
        practice(result, text)


def standard_hindi(pack, body):
    collected, current = [], None
    grade = 1
    for index, element in enumerate(body):
        if element.tag == W + "p":
            title = paragraph(element)
            style = element.find(f"{W}pPr/{W}pStyle")
            if title == "Current Session - कक्षा 2":
                grade = 2
            elif index >= 11 and style is not None and style.get(W + "val") == "Heading1":
                current = sheet(pack, grade, len(collected) + 1, title,
                    "पहले सुनाएँ, फिर बच्चे को बोलने, इशारा करने या लिखने दें। एक बार में एक निर्देश दें।")
                collected.append(current)
        elif current and element.tag == W + "tbl":
            rows = table(element)
            if not rows or not re.match(r"1\.\s*सुनो", rows[0][0]):
                continue
            for row in rows:
                for cell in row:
                    header, *lines = [line for line in cell.splitlines() if useful(line)]
                    for line in lines:
                        if "☐" in line:
                            prompt, *choices = line.split("☐")
                            opts = [option.strip() for option in choices]
                            known = {"किसका परिवार?": "मेरा"}
                            answer = known.get(prompt.strip())
                            if answer:
                                choice(current, prompt, opts, answer)
                            else:
                                practice(current, line, guidance=header)
                        else:
                            practice(current, line, guidance=header)
    return collected


def standard_math(pack, body):
    collected, current = [], None
    grade = 1
    for index, element in enumerate(body):
        if element.tag == W + "p":
            line = paragraph(element)
            if line.startswith("Part B"):
                grade = 2
            elif re.match(r"^(Bridge \d+|Chapter \d+:.*?) - Goal \d+:", line):
                current = sheet(pack, grade, len(collected) + 1, line)
                collected.append(current)
            elif current and line.startswith("Teacher/Parent prompt:"):
                current["adult"] = clean(line.split(":", 1)[1])
            elif current and line.startswith("□"):
                current["adult"] += "\n" + line.lstrip("□ ").strip()
        elif current and element.tag == W + "tbl":
            rows = table(element)
            if not rows:
                continue
            if rows[0][:2] == ["Task", "Response"]:
                for row in rows[1:]:
                    math_goal_task(current, row[0])
            elif rows[0][:3] == ["Column A", "Match", "Column B"]:
                if index == 16:
                    matching(current, [("under", "below"), ("inside", "in the box"), ("near", "close to")],
                             "Tap the words that mean the same.")
                else:
                    for first, _, second in rows[1:]:
                        practice(current, f"Match {first}.", "Options: " + ", ".join(row[2] for row in rows[1:]))
            elif rows[0][0].startswith("Write in the boxes"):
                for row in rows[1:]:
                    for value in row:
                        if value.isdigit():
                            add(current, "trace", value, "Trace the number, then write it.", model=value)
            elif rows[0][0].startswith(("Say and write", "Trace then write")):
                for row in rows:
                    for value in row:
                        if value.isdigit():
                            add(current, "trace", value, model=value)
    return collected


def math_goal_task(result, text):
    shapes = {"circle": "○", "square": "□", "triangle": "△", "rectangle": "▭"}
    if text == "Circle the ball inside the box.":
        choice(result, "Which picture has a ball inside the box?", ["⚽ ▢", "▣", "▢ ⚽"], "▣", hint="Look for the ball inside the box.")
    elif text in ("Circle all triangles.", "Tick all circles."):
        target = "triangle" if "triangles" in text else "circle"
        choice(result, f"Tap the {target}.", list(shapes.values()), shapes[target])
    elif text.startswith(("Trace a circle", "Trace a square", "Copy a triangle", "Complete a half-drawn rectangle")):
        shape = next(shape for shape in shapes if shape in text)
        add(result, "draw", text, shapes[shape])
    elif text == "Put these in order: wake up, brush, eat.":
        order(result, "Tap each morning action in order.", ["Wake up", "Brush teeth", "Eat breakfast"])
    elif text.startswith("Match can with cylinder"):
        choice(result, "What shape is a tin can?", ["Sphere", "Cube", "Cylinder"], "Cylinder", "🥫")
    elif text.startswith("Match square to window"):
        choice(result, "Tap the shape of a window with four equal sides.", list(shapes.values()), shapes["square"])
    elif text.startswith("Sphere has corners"):
        truth(result, ["A sphere has corners."], [False])
    elif text.startswith("Count corners on a box"):
        number(result, "How many corners are on a box?", 8, "📦")
    elif text.startswith("Circle the shape with no flat face"):
        choice(result, "Which shape has no flat face?", ["Cube", "Sphere", "Cylinder"], "Sphere")
    else:
        practice(result, text)

def detail_header(slug, row):
    combined = "\n".join(row)
    if slug == "math-activities":
        marker = re.search(r"Activity Sheet (\d+)", combined)
        return (int(marker[1]), 2 if "Class 2" in combined else 1, row[0].splitlines()[0]) if marker else None
    marker = re.search(r"\bWorksheet (\d+)\b", combined)
    if not marker:
        return None
    number = int(marker[1])
    if slug == "en-activities":
        return number, 1 if number <= 12 else 2, row[-1]
    return number, 1 if number <= 3 else 2, row[0].splitlines()[1]


def detailed(pack, body):
    slug = pack[0]
    collected, current = [], None
    pending = []
    for index, element in enumerate(body):
        if element.tag == W + "tbl":
            rows = table(element)
            if not rows:
                continue
            header = detail_header(slug, rows[0])
            if header:
                if current:
                    detail_tasks(slug, current, pending)
                current = sheet(pack, header[1], header[0], header[2])
                collected.append(current)
                pending = []
            elif current and not re.search(r"^(?:Name:|Name / नाम)", rows[0][0]):
                pending.append((index, rows))
        elif current and element.tag == W + "p" and (text := paragraph(element)):
            pending.append((index, text))
    if current:
        detail_tasks(slug, current, pending)
    return collected


def detail_tasks(slug, result, blocks):
    section, intro, reference = "", "", ""
    section_start = 0
    for index, block in blocks:
        if isinstance(block, str) and (heading := re.match(r"^[A-F] · (.+)", block)):
            if section and len(result["tasks"]) == section_start:
                practice(result, intro or section, reference)
            section, intro, reference = heading[1].capitalize(), "", ""
            section_start = len(result["tasks"])
        elif isinstance(block, list) and len(block) == 1 and len(block[0]) == 2 and re.fullmatch(r"[A-Z]", block[0][0]):
            if section and len(result["tasks"]) == section_start:
                practice(result, intro or section, reference)
            section, intro, reference = block[0][1], "", ""
            section_start = len(result["tasks"])
        elif (slug == "en-activities" and english_detail(result, index, block)) or (
            slug == "hi-activities" and hindi_detail(result, index, block)
        ) or (slug == "math-activities" and math_detail(result, index, block)):
            if index == 201 and slug == "en-activities":
                reference = "\n".join(row[0] for row in block)
        elif isinstance(block, list) and block[0][0].startswith("🌟"):
            if "word bank" not in block[0][0].lower():
                result["adult"] += "\n" + clean(block[0][0])
        elif isinstance(block, str):
            if not useful(block) or re.fullmatch(r"(?:Ans|Answer):\s*_+", block):
                continue
            if len(result["tasks"]) == section_start and not intro and "_" not in block and not re.match(r"^\d+\.", block):
                intro = block
                if not section and index == blocks[0][0]:
                    continue
            elif "word bank:" in block.lower() or re.match(r"^(?:Use:|Available coins/notes:|Labels to use:)", block):
                reference = block
            else:
                practice(result, block, reference, intro)
        else:
            content = [[cell for cell in row if useful(cell)] for row in block]
            active = [row for row in content if row]
            if not active:
                continue
            if all(re.fullmatch(r"(?:Ans|Answer|Draw here):\s*_*\s*", row[0], re.I) for row in active):
                continue
            if len(active) == 1 and len(active[0]) == 1 and len(active[0][0]) < 45 and "_" not in active[0][0]:
                if section and len(result["tasks"]) == section_start:
                    practice(result, intro or section, active[0][0])
                continue
            labels = active[0]
            if len(active) > 1 and any(re.search(
                r"^(?:Task|Word|Question|Answer|Sentence|Amount|Shape|Object|Type|Season|Month|Sport|Line(?:\s|\()|Activity|Number|Statement|Figure|#)\b",
                label, re.I) for label in labels
            ):
                active = active[1:]
            for row in active:
                shown = "  ·  ".join(row)
                if all(re.fullmatch(r"\d+", cell) for cell in row):
                    shown = ", ".join(row)
                practice(result, shown, reference or intro or section, section)
    if section and len(result["tasks"]) == section_start:
        practice(result, intro or section, reference)


def english_detail(result, index, block):
    if index in (8, 10, 15):
        if index == 8:
            matching(result, [(letter, letter.lower()) for letter in block[0]], "Tap the lowercase letter.")
        elif index == 15:
            for row in block:
                for cell in row:
                    if cell:
                        letter = cell[0]
                        add(result, "write", f"Write the lowercase letter for {letter}.", key=letter.lower())
        return True
    if index in (28, 41):
        for letter, examples, choices in block[1:]:
            options = choices.removeprefix("Circle: ").split()
            answer = next(option for option in options if option[0].casefold() == letter[0].casefold())
            choice(result, f"Which word starts like {letter[0]}?", options, answer, examples)
        return True
    if index == 54:
        for *words, _ in block[1:]:
            odd = [word for word in words if sum(other[0] == word[0] for other in words) == 1]
            if len(odd) == 1:
                choice(result, "Which word starts with a different sound?", words, odd[0])
            else:
                options = words + ["They all start the same"]
                choice(result, "Which word starts with a different sound?", options, options[-1], hint="Say each word slowly.")
        return True
    if index == 84:
        for word, *_ in block[1:]:
            add(result, "trace", word, "Trace, copy, and read the word.", model=word)
        return True
    if index == 96:
        parts = [row[0] for row in block[1:]]
        for part, sentence in block[1:]:
            choice(result, sentence, [part] + [item for item in parts if item != part][:3], part)
        return True
    if index in (113, 243, 309):
        prompt = {113: "Tap the meaning of the action.", 243: "Tap the opposite word.", 309: "Tap the animal’s action."}[index]
        matching(result, [tuple(row[:2]) for row in block[1:]], prompt)
        return True
    if index == 130:
        for (sentence,), answer in zip(block, ["This is", "These are"] * 5, strict=True):
            choice(result, strip_number(sentence), ["This is", "These are"], answer, hint="Think about one thing or more than one.")
        return True
    if index == 147:
        for (line,), answer in zip(block, ["up", "in", "down", "out", "on", "out", "", "in"], strict=True):
            if answer:
                choice(result, strip_number(line), ["up", "down", "in", "out", "on"], answer)
            else:
                add(result, "write", strip_number(line), "Use: up, down, in, out, on.")
        return True
    if index == 164:
        fruits = {"banana", "apple", "grapes", "mango", "guava", "orange"}
        for word, _, colour in block[1:]:
            choice(result, word, ["Fruit", "Vegetable"], "Fruit" if word in fruits else "Vegetable", "Sort the food.")
            shade = colour.strip("() ")
            choice(result, f"What colour is the {word} in this worksheet?", [shade] + [c for c in ("white", "green", "purple", "orange") if c != shade][:2], shade)
        return True
    if index == 179:
        for season, clue, answer in [
            ("Summer", "What does it feel like?", "Hot"),
            ("Winter", "What does it feel like?", "Cold"),
            ("Monsoon", "What can keep you dry?", "Umbrella"),
            ("Spring", "What might you see on trees?", "New leaves"),
        ]:
            choices = ["Hot", "Cold", "Mild"] if answer in ("Hot", "Cold") else ["Umbrella", "Warm cap", "New leaves"]
            choice(result, clue, choices, answer, season)
        return True
    if index == 211:
        poem = next(task["context"] for task in result["tasks"] if task["prompt"] == "Read the poem.")
        for (prompt,), answer in zip(block, ["red", "bell", "tyres and pedals", "air", "helmet"], strict=True):
            add(result, "write", strip_number(prompt), hint=poem, key=answer)
        return True
    if index == 263:
        for first, second, answer in block[1:]:
            add(result, "write", "Join the sentences with ‘and’.", first + "\n" + second,
                "Use the word ‘and’ to link the ideas.", answer if "_" not in answer else "")
        return True
    if index in (181, 186, 188):
        if index == 188:
            bands = [
                ("Summer", "hot", "holidays", "pool"),
                ("Winter", "cold", "woolen cap"),
                ("Monsoon", "umbrella", "rain", "puddles", "muddy roads"),
                ("Spring", "flowers bloom", "new leaves"),
            ]
            matching(result, [(word, season) for season, *words in bands for word in words], "Which season is a good match?")
            add(result, "write", "When do you see a colourful sky?", "Talk about the seasons you see where you live.")
        return True
    if index == 201:
        add(result, "adult", "Read the poem.", "\n".join(row[0] for row in block))
        return True
    if index == 206:
        for claim, _, key in block[1:]:
            choice(result, claim, ["True", "False"], "True" if "✓" in key else "False",
                   "Think about the poem My Bicycle.", "Read the line in the poem again.")
        return True
    if index in (221, 346):
        for row in block:
            for cell in row:
                text, answer = cell.rsplit("(", 1)
                choice(result, text, ["a", "an"], answer.strip(") "), hint="Listen to the first sound.")
        return True
    if index in (226, 327, 359):
        if index == 226:
            words = block.split(":", 1)[1].split()
            matching(result, [(word, "an" if word[0] in "aeiou" else "a") for word in words], "Which article goes with the word?")
        elif index == 327:
            words = [item.strip() for item in block.split(":", 1)[1].split("·")]
            matching(result, [(word, f"Short {next(letter for letter in word if letter in 'eio')}") for word in words], "Tap the vowel sound.")
        else:
            words = [item.strip() for item in block.split(":", 1)[1].split("·")]
            air = {"aeroplane", "helicopter", "rocket"}
            water = {"ship", "boat", "yacht", "raft"}
            matching(result, [(word, "Air" if word in air else "Water" if word in water else "Land") for word in words], "Where does it travel?")
        return True
    if index in (228, 331, 363):
        return True
    if index == 280:
        events = [row[1] for row in block]
        order(result, "Put the story events in order.", [events[i] for i in (6, 1, 3, 0, 2, 4, 5, 7)], shown=events)
        return True
    if index in (396, 401, 416):
        for row in block:
            for cell in row:
                prompt, key = cell.rsplit("(", 1)
                key = key.strip(") ")
                options = ["bl", "pl", "fl", "cl"] if index == 416 else ["ch", "sh"]
                solution = key if index == 416 else "ch" if index == 396 else "sh"
                choice(result, prompt, options, solution, hint="Say the whole word.")
        return True
    if index == 431:
        for word, split, count in block[1:]:
            choice(result, f"How many syllables in “{word}”?", ["1", "2", "3", "4"], count, "Clap once per syllable.", f"Clap as you say {split}.")
        return True
    if index == 455:
        for original, answer in block[1:]:
            add(result, "write", strip_number(original).replace(" / ", "\n"), "Join the ideas using ‘but’.",
                key=answer.replace(" i ", " I "))
        return True
    if index == 460:
        for jumbled, answer in block[1:]:
            words = strip_number(jumbled).split(" / ")
            correct = answer.rstrip(".").split()
            by_word = {item.lower(): item for item in words}
            if len(by_word) == len(words) and {item.lower() for item in correct} == set(by_word):
                order(result, "Tap the words in sentence order.", [by_word[item.lower()] for item in correct], shown=words)
            else:
                add(result, "write", "Put the words in sentence order.", " / ".join(words), key=answer)
        return True
    return False


HINDI_MATCH = {
    4: [3, 0, 1, 2, 4],
    35: [2, 0, 3, 1],
    71: [1, 0, 3, 2],
    84: [3, 0, 4, 2, 1],
    110: [1, 2, 0, 4, 3],
}
HINDI_BANKS = [
    (range(9, 13), ["हाथी", "बकरी", "चिड़िया", "मछली", "गाय"], ["मछली", "हाथी", "चिड़िया", "गाय"]),
    (range(44, 48), ["रोटी", "दूध", "सब्जी", "फल", "मिठाई"], ["रोटी", "दूध", "फल", "मिठाई"]),
    (range(89, 93), ["दादी", "माँ", "पिता", "भाई", "घर"], ["दादी", "माँ", "पिता", "भाई"]),
    (range(115, 119), ["लाल", "पीला", "हरा", "नीला", "सफेद", "बैंगनी"], ["नीला", "लाल", "पीला", "हरा"]),
    (range(151, 155), ["पहले", "फिर", "उसके बाद", "आखिर में"], ["पहले", "फिर", "उसके बाद", "आखिर में"]),
    (range(177, 181), ["क्यों", "कैसे", "कब", "कहाँ", "कौन"], ["कौन", "कहाँ", "कब", "कैसे"]),
]


def hindi_detail(result, index, block):
    if index in HINDI_MATCH:
        options = [re.sub(r"^[a-z]\.\s*", "", row[1]) for row in block]
        pairs = [(strip_number(row[0]), options[position]) for row, position in zip(block, HINDI_MATCH[index], strict=True)]
        matching(result, pairs, "सही जोड़ी छुओ।", options)
        return True
    for positions, options, keys in HINDI_BANKS:
        if index in positions:
            choice(result, strip_number(block), options, keys[positions.index(index)], "सही शब्द छुओ।")
            return True
    if index in (8, 43, 88, 114, 150, 176, 206):
        return True
    if index == 39:
        for plain, mark, *_ in block[1:]:
            key = re.search(r"→\s*(\S+)", mark)
            model = key[1] if key else plain
            add(result, "trace", model, f"मात्रा लगाकर शब्द लिखो।  {plain}", model=model)
        return True
    if index == 63:
        answers = iter(["ी", "े", "ं", "ू", "ी", "ु"])
        for row in block[1:]:
            for word, options in zip(row[::2], row[1::2], strict=True):
                choice(result, word, ["◌" + option.strip() for option in options.split("·")], "◌" + next(answers), "कौन-सी मात्रा इस शब्द में है?")
        return True
    if index == 66:
        pairs = [
            ("दीया", "ी / ि"), ("मेला", "े / ो"), ("रोटी", "े / ो"), ("गुड़", "ु / ू"),
            ("पूजा", "ु / ू"), ("खेल", "े / ो"), ("मिठाई", "ी / ि"), ("धूप", "ु / ू"),
        ]
        for word, target in pairs:
            choice(result, word, ["◌ा", "◌ी / ◌ि", "◌ु / ◌ू", "◌े / ◌ो"], "◌" + target.replace("/ ", "/ ◌"),
                   "शब्द में इस्तेमाल हुई एक मात्रा चुनो।")
        return True
    if index == 67:
        return True
    if index == 96:
        for (jumbled, _), key in zip(block[1:], ["दादी", "नाना", "मामा", "पिता"], strict=True):
            add(result, "write", jumbled, "अक्षर सही क्रम में लिखो।", key=key)
        return True
    if index in (138, 140, 142, 144, 146):
        if index == 138:
            growing = ["किसान ने बीज चुना।", "बीज को ज़मीन में डाला।", "हर रोज पानी दिया।", "छोटा पौधा निकला।", "पौधे में फूल खिले।"]
            order(result, "बीज कैसे उगा? सही क्रम में छुओ।", growing)
        return True
    if index == 157:
        words = [word.strip() for word in block.split("Words: ", 1)[1].split("·")]
        living = {"पेड़", "मछली", "तितली", "बीज"}
        matching(result, [(word, "सजीव" if word in living else "निर्जीव") for word in words], "सजीव या निर्जीव?")
        return True
    if index in (172, 212):
        truth(result, [row[0] for row in block],
            [True, False, True, False, True] if index == 172 else [False, True, True, False], hindi=True)
        return True
    if index == 202:
        matching(result, [("↑ North", "उत्तर"), ("← West", "पश्चिम"), ("→ East", "पूर्व"), ("↓ South", "दक्षिण")],
                 "दिशा का हिंदी नाम चुनो।")
        return True
    if index in (207, 208, 209):
        key = {207: "चाँद और तारे", 208: "बादल से बारिश", 209: "सूरज"}[index]
        add(result, "write", strip_number(block), "बादल · चाँद · तारे · सूरज · बारिश", key=key)
        return True
    return False

def math_detail(result, index, block):
    if index in (6, 10, 134):
        for row in block[1:]:
            groups = [row[pos:pos + 3] for pos in range(0, min(len(row), 6 if index == 134 else len(row)), 3)]
            for first, second, _ in groups:
                if index == 10:
                    number(result, f"{first} tens and {second} ones make what number?", int(first) * 10 + int(second))
                else:
                    tens, ones = divmod(int(first), 10)
                    number(result, f"How many tens are in {first}?", tens)
                    number(result, f"How many ones are in {first}?", ones)
            if index == 134:
                number_value = int(row[6])
                tens, ones = divmod(number_value, 10)
                add(result, "write", f"Write {number_value} in expanded form.", key=f"{tens * 10} + {ones}")
        return True
    if index == 14:
        for row in block:
            for position in range(0, len(row), 4):
                left, right = int(row[position]), int(row[position + 2])
                choice(result, f"{left} ___ {right}", ["<", "=", ">"], compare(left, right), "Compare the numbers.")
        return True
    if index in (18, 20, 41, 45, 49):
        values = block[0]
        known_index = next(i for i, cell in enumerate(values) if cell.isdigit())
        step = {41: 2, 45: 5, 49: 10}.get(index, 1)
        start = int(values[known_index]) - known_index * step
        for i, value in enumerate(values):
            if not value:
                line = "  ·  ".join(cell or ("___" if j == i else "□") for j, cell in enumerate(values))
                number(result, "Which number goes in this blank?", start + i * step, line)
        return True
    if index in (32, 34, 36):
        symbols = block[0][0].split()
        length = next(n for n in range(1, len(symbols)) if all(symbol == symbols[i % n] for i, symbol in enumerate(symbols)))
        pattern = symbols[:length]
        for i in range(3):
            answer = pattern[len(symbols) % length]
            choice(result, "Which shape comes next?", list(dict.fromkeys(symbols)) + ["◇"], answer, "  ".join(symbols) + "  ___")
            symbols.append(answer)
        return True
    if index == 24:
        story = "\n".join(block[0][0].splitlines()[:2])
        choice(result, "Who has more sticks?", ["Raju", "Priya"], "Raju", story)
        number(result, "How many more sticks?", 15, story)
        return True
    if index in (57, 58):
        known = [part.strip() for part in re.split(r",", block.split("   ", 1)[1])]
        start, step = (4, 3) if index == 57 else (100, -10)
        for i, value in enumerate(known):
            if "_" in value:
                number(result, "Which number goes in the blank?", start + i * step, 
                       ", ".join(v if "_" not in v else ("___" if i == j else "□") for j, v in enumerate(known)))
        return True
    if index == 66:
        order(result, "Start with Monday. Tap the days of the week in order.", weekdays())
        return True
    if index == 70:
        left = [row[1] for row in block]
        right = [row[3] for row in block]
        order(result, "Put the morning, school, and evening actions in order.", [left[i] for i in (1, 0, 2, 4, 3)], shown=left)
        order(result, "Put these actions in order.", [right[i] for i in (0, 1, 2, 3, 4)], shown=right)
        return True
    if index == 74:
        days = weekdays()
        for day, *_ in block[1:]:
            start = days.index(day)
            for question, offset in (("What day was yesterday?", -1), ("What day is tomorrow?", 1), ("What day is after tomorrow?", 2)):
                choice(result, question, [days[(start + offset + i) % 7] for i in (-2, 0, 1)], days[(start + offset) % 7], f"Today is {day}.")
        return True
    if index == 78:
        periods = ["Morning", "Night", "Midday", "Evening", "Night"]
        matching(result, [(row[0], answer) for row, answer in zip(block[1:], periods, strict=True)],
                 "Which time of day matches?", [row[2] for row in block[1:]])
        return True
    if index in (86, 90):
        for row in block[1:] if index == 90 else block:
            group, objects = [int(value) for value in re.findall(r"\d+", row[0])]
            if index == 90:
                add(result, "write", f"Write the repeated addition for {row[0]}.", key=" + ".join([str(objects)] * group))
            number(result, "How many altogether?", group * objects, row[0])
        return True
    if index in (94, 272):
        for question, *_ in block[1:]:
            rows, cols = [int(value) for value in re.findall(r"\d+", question)]
            add(result, "draw", f"Draw {question}.")
            number(result, f"Complete the sentence: {rows} × {cols} = ___", rows * cols)
        return True
    if index in (98, 99, 100, 119, 120, 121, 251, 252, 253, 254):
        prompt = block[0][0] if isinstance(block, list) else re.sub(r"\s+Answer:.*|\s+_+$", "", block)
        answer = {98: 20, 99: 18, 100: 16, 119: 0, 120: 14, 121: 5, 251: 27, 252: 5, 253: 21, 254: 22}[index]
        context = "Shop prices: Pencil ₹4 · Eraser ₹6 · Sharpener ₹8 · Notebook ₹15." if index in (119, 120, 121) else ""
        number(result, prompt, answer, context)
        return True
    if index == 117:
        return True
    if index == 122:
        add(result, "write", block, "Shop prices: Pencil ₹4 · Eraser ₹6 · Sharpener ₹8 · Notebook ₹15.",
            key="Two erasers cost ₹12. A pencil and a sharpener also cost ₹12.")
        return True
    if index == 109:
        for value in block[0]:
            number(result, "What is the amount in rupees?", int(value.strip("₹")), value)
        return True
    if index in (126, 317):
        for amount, *_ in block[1:]:
            add(result, "write", f"Make {amount} using coins or notes.",
                "Find two different ways." if index == 126 else "Use ₹1, ₹2, ₹5, ₹10, ₹20, ₹50, or ₹100. Count how many you use.")
        return True
    if index in (146, 147, 148):
        nums = block[0][0].split(", ")
        order(result, "Tap from smallest to largest.", sorted(nums, key=int), shown=nums)
        return True
    if index == 153:
        for (prompt, a, _, b, _), answer in zip(block[1:], ("10", "7", "12", ""), strict=True):
            if answer:
                choice(result, prompt, [a, b], answer, "Which is the closer estimate?")
            else:
                add(result, "adult", prompt, f"Choose the closer estimate with a grown-up: {a} or {b}.")
        return True
    if index == 161:
        for (shape, *_), counts in zip(block[1:], ((6,12,8),(6,12,8),(0,0,0),(2,2,0),(1,1,1)), strict=True):
            add(result, "write", f"Write the name of something shaped like a {shape.lower()}.")
            for part, count in zip(("flat faces", "edges", "corners"), counts, strict=True):
                number(result, f"How many {part} does a {shape.lower()} have?", count)
        return True
    if index == 165:
        shapes = ["Sphere", "Cube", "Cylinder", "Cone", "Cuboid", "Sphere"]
        matching(result, [(name, answer) for name, answer in zip(
            [row[position] for row in block[1:] for position in (0, 2)], shapes, strict=True)], "Tap the matching shape.")
        return True
    if index in (169, 193):
        values = [True, True, False, True, False, True] if index == 169 else [True, False, True, False, True, True]
        if index == 169:
            truth(result, [row[0] for row in block[1:]], values)
        else:
            for row, value in zip(block[1:], values, strict=True):
                choice(result, row[0], ["Open", "Closed"], "Closed" if value else "Open")
        return True
    if 173 <= index <= 176:
        shape = ["Cube", "Cylinder", "Sphere", "Cone"][index - 173]
        choice(result, block, ["Cube", "Cylinder", "Sphere", "Cone"], shape)
        return True
    if index == 184:
        for (shape, *_), sides in zip(block[1:], (0, 4, 3, 4, 5), strict=True):
            add(result, "draw", f"Draw a {shape.lower()}.")
            number(result, f"How many straight sides does a {shape.lower()} have?", sides)
            number(result, f"How many corners does a {shape.lower()} have?", sides)
            add(result, "write", f"Write something special about a {shape.lower()}.")
        return True
    if index == 189:
        prompts = [(row[0], row[2].split("-->")[0].strip()) for row in block]
        keys = ("Vertical", "Horizontal", "Slanting", "Curved", "Closed", "Straight")
        options = [("Vertical", "Horizontal", "Slanting"), ("Vertical", "Horizontal", "Slanting"),
                   ("Vertical", "Horizontal", "Slanting"), ("Straight", "Curved"), ("Open", "Closed"), ("Straight", "Curved")]
        for prompt, answer, choices in zip([item for row in prompts for item in row], keys, options, strict=True):
            choice(result, prompt, choices, answer)
        return True
    if index in (201, 205, 223, 227):
        for cell in block[0]:
            digits = re.findall(r"\d", cell)
            if len(digits) == 4:
                left, right = ("".join(digits[:2]), "".join(digits[2:]))
                op = "+" if index < 210 else "-"
                arithmetic(result, f"{left} {op} {right}")
        return True
    if index in (209, 231, 283):
        for row, answer in zip(block, {209: (59,83,97),231: (38,38,37),283: (30,6,28)}[index], strict=True):
            if index == 283:
                choice(result, "Which operation will you use?", ["Multiply", "Divide"],
                       "Divide" if answer == 6 else "Multiply", row[0])
            add(result, "write", "Show your working.", row[0])
            number(result, row[0], answer)
        return True
    if index in (213, 214):
        for cell in block[0]:
            if cell:
                known, total = [int(value) for value in re.findall(r"\d+", cell)]
                number(result, cell, total - known)
        return True
    if index == 243:
        for row in block[1:]:
            if row[1].isdigit():
                add(result, "adult", f"Line {row[1]}: Estimate the length, then measure it.",
                    "Ask a grown-up to draw a line on paper. Use a real ruler.")
        return True
    if index == 247:
        items = block[0][::2]
        order(result, "Tap the objects from shortest to longest.", sorted(items, key=lambda item: int(re.search(r"\d+", item)[0])),
              shown=items)
        return True
    if index in (258, 259, 260):
        opts = re.split(r" {2,}", block)
        options = [x[1:-1] for x in opts[1:]]
        choice(result, opts[0], options, options[1 if index != 259 else 2])
        return True
    if index in (235, 268, 113):
        for row in block[1:] if index == 268 else block:
            for cell in row:
                if cell:
                    arithmetic(result, cell)
        return True
    if 276 <= index <= 279:
        total, groups = [int(value) for value in re.findall(r"\d+", block)]
        number(result, block, total // groups)
        return True
    if index == 291:
        months = [row[pos] for pos in (1, 4) for row in block[1:]]
        for start in (0, 6):
            order(result, f"Tap the months in order, starting with {months[start]}.", months[start:start + 6])
        for month in months:
            add(result, "adult", f"What season is {month} where you live?")
        return True
    if index in (295, 296, 297, 298):
        add(result, "adult", block, "Look at this month on a calendar with a grown-up.")
        return True
    if index == 299:
        choice(result, block, ["December", "January", "February"], "January")
        return True
    if index == 300:
        number(result, block, 4)
        return True
    if index == 304:
        for cell in block[0]:
            value = re.search(r"(\d+):(\d+)", cell)
            hour, minute = (int(part) for part in value.groups())
            codepoint = (0x1F550 if minute == 0 else 0x1F55C) + hour - 1
            shown = f"{hour}:{minute:02}"
            options = [shown, f"{hour}:{(minute + 30) % 60:02}", f"{hour % 12 + 1}:{minute:02}"]
            choice(result, "What time does the clock show?", options, shown, chr(codepoint))
        for cell in block[1]:
            add(result, "draw", "Draw the clock hands for " + cell.splitlines()[-1] + ".", "◯")
        return True
    if index == 308:
        for (activity, start, end, _), duration in zip(block[1:], (30,60,30,60), strict=True):
            number(result, f"How many minutes is {activity.lower()}?", duration, f"{start} → {end}")
        return True
    if index in (316, 321):
        return True
    if 323 <= index <= 326:
        prices = "Balloon ₹8 · Popcorn ₹15 · Candy ₹5 · Juice ₹20 · Toy ₹35"
        item, _, _ = block.partition("  ")
        total, change = [(13,7), (35,15), (40,10), (83,None)][index - 323]
        number(result, item + " What is the total?", total, prices)
        if change is not None:
            paid = (20, 50, 50)[index - 323]
            number(result, f"How much change from ₹{paid}?", change, f"Total price ₹{total}")
        return True
    if index == 330:
        for (question, *_), answer in zip(block, (37, 30, 35), strict=True):
            add(result, "write", "Show your working.", question)
            if answer == 35:
                number(result, question.split(" Can")[0], answer)
                choice(result, "Can Priya buy a toy for ₹30?", ["Yes", "No"], "Yes", "Priya saved ₹35.")
            else:
                number(result, question, answer)
        return True
    if index in (338, 339):
        return True
    if index in (340, 344, 348, 349, 350, 351, 352, 356):
        data = "Cricket 9 · Football 6 · Kabaddi 5 · Kho-kho 4 · Badminton 4"
        if index in (340, 344):
            for row in block[1:]:
                name = row[0]
                add(result, "draw", f"Draw the {'tally marks' if index == 340 else 'stars'} for {name}.", data)
        elif index == 348:
            choice(result, block, ["Cricket", "Football", "Kabaddi"], "Cricket", data)
        elif index == 349:
            choice(result, block, ["Cricket", "Kabaddi", "Kho-kho and Badminton"], "Kho-kho and Badminton", data)
        elif index in (350, 351):
            number(result, block, 3 if index == 350 else 9, data)
        elif index == 352:
            choice(result, block, ["Running games", "Bat-and-ball games"], "Bat-and-ball games", data)
        else:
            add(result, "draw", "Draw a bar chart. Use one square for each student.", data)
        return True
    return False


def weekdays():
    return ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("inputs", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    builder = {
        "en-chapters": standard_english,
        "hi-chapters": standard_hindi,
        "math-chapters": standard_math,
    }
    combined = []
    for pack in PACKS:
        source = outline(args.inputs / (pack[3] + ".docx"))
        sheets = builder.get(pack[0], detailed)(pack, source)
        for item in sheets:
            item["adult"] = item["adult"].strip()
            assert item["tasks"], f'No exercises found in {item["title"]}'
        print(f"{pack[0]}: {len(sheets)} worksheets, {sum(len(item['tasks']) for item in sheets)} tasks")
        combined.extend(sheets)
    args.output.write_text(json.dumps(combined, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
