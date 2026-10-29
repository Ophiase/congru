pub const STYLE: &str = r"<style>
.caption { font-size: 13px; fill: #3e4c59; }
.stroke { fill: none; stroke-width: 1.8; stroke-linecap: round; }
.model { stroke: #e03131; fill: #e03131; }
.observed { stroke: #1c7ed6; fill: #1c7ed6; }
.outlier { stroke: #2f9e44; fill: #2f9e44; }
.faint { stroke: #adb5bd; fill: #adb5bd; }
.link { stroke: #868e96; stroke-width: 0.8; stroke-dasharray: 2 2; }
.label { font-size: 9px; fill: #1c7ed6; }
.ring { fill: #e03131; fill-opacity: 0.08; stroke: #e03131; stroke-opacity: 0.7; stroke-width: 1.1; stroke-dasharray: 4 3; }
@media (prefers-color-scheme: dark) {
  .caption { fill: #ced4da; }
  .model { stroke: #ff8787; fill: #ff8787; }
  .observed { stroke: #4dabf7; fill: #4dabf7; }
  .outlier { stroke: #69db7c; fill: #69db7c; }
  .faint { stroke: #495057; fill: #495057; }
  .link { stroke: #868e96; }
  .label { fill: #4dabf7; }
  .ring { fill: #ff8787; stroke: #ff8787; }
}
</style>
";
