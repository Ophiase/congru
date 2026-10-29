# Naming

**[question]** The repository is named *Efficient Point Cloud Alignement*
(French spelling). Before publishing, a short name for the crate, the Python
package and the C prefix.

| Candidate | For | Against |
| --- | --- | --- |
| `recalage` | The French word for registration; free, short, honest | Not obvious to an English reader |
| `align3d` | Says what it does | Generic, likely taken somewhere |
| `rigid` | Says the hypothesis | Too generic to search for |
| `kabsch` | Known to the field | Names one method of many |

The draft code uses `recalage` as a working name; a rename is one search and
replace until the first release.
