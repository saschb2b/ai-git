import { useEffect, useRef } from "react";
import { useNavigate } from "react-router";
import Box from "@mui/material/Box";
import Typography from "@mui/material/Typography";
import CircularProgress from "@mui/material/CircularProgress";
import Alert from "@mui/material/Alert";
import * as d3 from "d3";
import { useApi } from "../hooks/useApi";

interface GraphNode extends d3.SimulationNodeDatum {
  id: string;
  description: string;
  status: "active" | "closed";
}

interface GraphEdge {
  source: string;
  target: string;
}

interface GraphData {
  nodes: GraphNode[];
  edges: GraphEdge[];
}

export function GraphPage() {
  const { data, loading, error } = useApi<GraphData>("/api/graph");
  const svgRef = useRef<SVGSVGElement>(null);
  const navigate = useNavigate();

  useEffect(() => {
    if (!data || !svgRef.current) return;

    const svg = d3.select(svgRef.current);
    svg.selectAll("*").remove();

    const width = svgRef.current.clientWidth;
    const height = svgRef.current.clientHeight;

    const g = svg.append("g");

    // Zoom
    svg.call(
      d3
        .zoom<SVGSVGElement, unknown>()
        .scaleExtent([0.2, 4])
        .on("zoom", (event) => {
          g.attr("transform", event.transform);
        }),
    );

    const nodes: GraphNode[] = data.nodes.map((n) => ({ ...n }));
    const links = data.edges.map((e) => ({ ...e }));

    const simulation = d3
      .forceSimulation(nodes)
      .force(
        "link",
        d3
          .forceLink(links)
          .id((d) => (d as GraphNode).id)
          .distance(120),
      )
      .force("charge", d3.forceManyBody().strength(-300))
      .force("center", d3.forceCenter(width / 2, height / 2));

    // Edges
    const link = g
      .append("g")
      .selectAll("line")
      .data(links)
      .join("line")
      .attr("stroke", "rgba(100, 108, 255, 0.3)")
      .attr("stroke-width", 1.5);

    // Nodes
    const node = g
      .append("g")
      .selectAll<SVGCircleElement, GraphNode>("circle")
      .data(nodes)
      .join("circle")
      .attr("r", 8)
      .attr("fill", (d) => (d.status === "active" ? "#646cff" : "#484f58"))
      .attr("stroke", (d) =>
        d.status === "active" ? "#8b91ff" : "#30363d",
      )
      .attr("stroke-width", 2)
      .style("cursor", "pointer")
      .on("click", (_, d) => {
        navigate(`/intents/${d.id}`);
      })
      .call(
        d3
          .drag<SVGCircleElement, GraphNode>()
          .on("start", (event, d) => {
            if (!event.active) simulation.alphaTarget(0.3).restart();
            d.fx = d.x;
            d.fy = d.y;
          })
          .on("drag", (event, d) => {
            d.fx = event.x;
            d.fy = event.y;
          })
          .on("end", (event, d) => {
            if (!event.active) simulation.alphaTarget(0);
            d.fx = null;
            d.fy = null;
          }),
      );

    // Labels
    const label = g
      .append("g")
      .selectAll("text")
      .data(nodes)
      .join("text")
      .text((d) =>
        d.description.length > 30
          ? d.description.slice(0, 30) + "..."
          : d.description,
      )
      .attr("font-size", 11)
      .attr("fill", "#c9d1d9")
      .attr("dx", 14)
      .attr("dy", 4);

    // Tooltip
    node.append("title").text((d) => d.description);

    simulation.on("tick", () => {
      link
        .attr("x1", (d) => (d.source as unknown as GraphNode).x!)
        .attr("y1", (d) => (d.source as unknown as GraphNode).y!)
        .attr("x2", (d) => (d.target as unknown as GraphNode).x!)
        .attr("y2", (d) => (d.target as unknown as GraphNode).y!);

      node.attr("cx", (d) => d.x!).attr("cy", (d) => d.y!);
      label.attr("x", (d) => d.x!).attr("y", (d) => d.y!);
    });

    return () => {
      simulation.stop();
    };
  }, [data, navigate]);

  if (loading) {
    return (
      <Box sx={{ display: "flex", justifyContent: "center", py: 8 }}>
        <CircularProgress />
      </Box>
    );
  }

  if (error) {
    return <Alert severity="error">{error}</Alert>;
  }

  if (!data || data.nodes.length === 0) {
    return (
      <Typography color="text.secondary" sx={{ py: 4, textAlign: "center" }}>
        No intents yet.
      </Typography>
    );
  }

  return (
    <Box
      sx={{
        width: "100%",
        height: "calc(100vh - 140px)",
        bgcolor: "background.paper",
        borderRadius: 2,
        overflow: "hidden",
      }}
    >
      <svg ref={svgRef} width="100%" height="100%" />
    </Box>
  );
}
