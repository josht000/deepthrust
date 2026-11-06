import csv
import zipfile

from tools.build_form4_dataset import build, mentions_10b5_plan


def _write_tsv(archive, name, rows):
    headers = list(rows[0])
    body = "\t".join(headers) + "\n"
    body += "".join("\t".join(row.get(key, "") for key in headers) + "\n" for row in rows)
    archive.writestr(name, body)


def test_10b5_1_footnote_detection_includes_common_spellings_and_negation():
    assert mentions_10b5_plan("Sale pursuant to a Rule 10b5-1 plan.")
    assert mentions_10b5_plan("Transaction under 10(b)(5)-1 trading plan.")
    assert not mentions_10b5_plan("The transaction was not made pursuant to a 10b5-1 plan.")


def test_form4_build_joins_footnotes_and_calculates_prior_same_month_years(tmp_path):
    incoming, output = tmp_path / "quarterly", tmp_path / "normalized"
    incoming.mkdir()
    submissions = []
    owners = []
    transactions = []
    footnotes = []
    for year in range(2006, 2011):
        accession = f"0000000001-{year}-000001"
        filed = "16-JUN-2010" if year == 2008 else ("16-JUL-2010" if year == 2010 else f"16-MAR-{year}")
        submissions.append(
            {
                "ACCESSION_NUMBER": accession,
                "FILING_DATE": filed,
                "DOCUMENT_TYPE": "4",
                "ISSUERCIK": "0000000123",
                "ISSUERTRADINGSYMBOL": '"(ABC)"',
                "REMARKS": "",
            }
        )
        owners.append(
            {
                "ACCESSION_NUMBER": accession,
                "RPTOWNERCIK": "0000000009",
            }
        )
        transactions.append(
            {
                "ACCESSION_NUMBER": accession,
                "TRANS_DATE": f"{year}-03-15",
                "TRANS_CODE": "P",
                "TRANS_SHARES": "100",
                "TRANS_PRICEPERSHARE": "10",
                "SECURITY_TITLE_FN": "F1" if year == 2006 else "",
            }
        )
        if year == 2006:
            footnotes.append(
                {
                    "ACCESSION_NUMBER": accession,
                    "FOOTNOTE_ID": "F1",
                    "FOOTNOTE_TXT": "Purchase pursuant to a 10b5-1 plan.",
                }
            )

    archive_path = incoming / "2006q1_form345.zip"
    with zipfile.ZipFile(archive_path, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        _write_tsv(archive, "SUBMISSION.tsv", submissions)
        _write_tsv(archive, "REPORTINGOWNER.tsv", owners)
        _write_tsv(archive, "NONDERIV_TRANS.tsv", transactions)
        _write_tsv(archive, "FOOTNOTES.tsv", footnotes or [{"ACCESSION_NUMBER": "", "FOOTNOTE_ID": "", "FOOTNOTE_TXT": ""}])

    build(incoming, output)
    with (output / "form4_owner_events.csv").open(newline="", encoding="utf-8") as source:
        rows = list(csv.DictReader(source))
    assert len(rows) == 5
    assert rows[0]["ticker"] == "ABC"
    assert rows[0]["buy_eligible"] == "0"
    assert rows[-1]["buy_eligible"] == "1"
    year_2009 = next(row for row in rows if row["filing_date"] == "2009-03-16")
    year_2010 = next(row for row in rows if row["filing_date"] == "2010-07-16")
    assert year_2009["buy_prior_same_month_years"] == "0"
    assert year_2010["buy_prior_same_month_years"] == "4"
    assert rows[0]["tenb5_sources"] == "footnote"
