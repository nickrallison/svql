#!/bin/sh

yosys scripts/yosys/graph.ys
dot -Tpdf design_graph.dot -o design_graph.pdf