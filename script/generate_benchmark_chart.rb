#!/usr/bin/env ruby
# frozen_string_literal: true

require 'csv'
require 'date'
require 'fileutils'
require 'graphviz'

# Ensure output directory exists
output_dir = ARGV[1] || 'benchmark-results'
FileUtils.mkdir_p(output_dir)

# Input file (benchmark history CSV)
input_file = ARGV[0] || 'benchmark-results/benchmark-history.csv'

unless File.exist?(input_file)
  puts "Error: File #{input_file} does not exist."
  exit 1
end

# Read benchmark data
data = CSV.read(input_file, headers: true)

# Limit to the last 10 entries to keep the chart readable
data = data.last(10) if data.size > 10

# Extract data points
dates = data.map { |row| row['date'] }
commits = data.map { |row| row['commit'][0..7] } # Short commit hash
curb_time = data.map { |row| row['curb_time'].to_f }
http_time = data.map { |row| row['http_time'].to_f }
rquest_time = data.map { |row| row['rquest_time'].to_f }
curb_rps = data.map { |row| row['curb_req_per_sec'].to_f }
http_rps = data.map { |row| row['http_req_per_sec'].to_f }
rquest_rps = data.map { |row| row['rquest_req_per_sec'].to_f }

# Generate chart for request time (lower is better)
time_graph = GraphViz.new(:G, type: :digraph) do |g|
  g[:rankdir] = 'LR'
  g[:bgcolor] = 'transparent'
  g.node[:shape] = 'none'
  g.node[:fontname] = 'Arial'
  g.edge[:fontname] = 'Arial'
  g.edge[:style] = 'invis'
  g[:ranksep] = '0.1'
  g[:label] = 'Request Time Comparison (seconds, lower is better)'
  g[:labelloc] = 't'
  
  # Create nodes for each data point
  prev_node = nil
  commits.each_with_index do |commit, i|
    # Label with commit hash and date
    label = "#{commit}\\n#{dates[i]}"
    
    # Create HTML-like label with a table to show values
    html_label = <<~HTML
      <TABLE BORDER="0" CELLBORDER="0" CELLSPACING="0" CELLPADDING="4">
        <TR><TD COLSPAN="2" ALIGN="CENTER">#{label}</TD></TR>
        <TR><TD ALIGN="RIGHT">Curb:</TD><TD BGCOLOR="#FF9999" WIDTH="#{(curb_time[i] * 50).to_i}"> #{curb_time[i].round(2)}s</TD></TR>
        <TR><TD ALIGN="RIGHT">HTTP.rb:</TD><TD BGCOLOR="#99CCFF" WIDTH="#{(http_time[i] * 50).to_i}"> #{http_time[i].round(2)}s</TD></TR>
        <TR><TD ALIGN="RIGHT">Rquest-rb:</TD><TD BGCOLOR="#99FF99" WIDTH="#{(rquest_time[i] * 50).to_i}"> #{rquest_time[i].round(2)}s</TD></TR>
      </TABLE>
    HTML
    
    node = g.add_nodes("data#{i}", label: html_label)
    
    # Connect nodes in sequence
    g.add_edges(prev_node, node) if prev_node
    prev_node = node
  end
  
  # Add a legend
  legend_html = <<~HTML
    <TABLE BORDER="0" CELLBORDER="0" CELLSPACING="0" CELLPADDING="4">
      <TR><TD COLSPAN="2" ALIGN="CENTER"><B>Legend</B></TD></TR>
      <TR><TD BGCOLOR="#FF9999" WIDTH="20"></TD><TD>Curb</TD></TR>
      <TR><TD BGCOLOR="#99CCFF" WIDTH="20"></TD><TD>HTTP.rb</TD></TR>
      <TR><TD BGCOLOR="#99FF99" WIDTH="20"></TD><TD>Rquest-rb</TD></TR>
    </TABLE>
  HTML
  
  g.add_nodes("legend", label: legend_html)
end

# Generate chart for requests per second (higher is better)
rps_graph = GraphViz.new(:G, type: :digraph) do |g|
  g[:rankdir] = 'LR'
  g[:bgcolor] = 'transparent'
  g.node[:shape] = 'none'
  g.node[:fontname] = 'Arial'
  g.edge[:fontname] = 'Arial'
  g.edge[:style] = 'invis'
  g[:ranksep] = '0.1'
  g[:label] = 'Requests Per Second Comparison (higher is better)'
  g[:labelloc] = 't'
  
  # Find max value for normalization
  max_rps = [curb_rps.max, http_rps.max, rquest_rps.max].max
  scale_factor = 200.0 / max_rps
  
  # Create nodes for each data point
  prev_node = nil
  commits.each_with_index do |commit, i|
    # Label with commit hash and date
    label = "#{commit}\\n#{dates[i]}"
    
    # Create HTML-like label with a table to show values
    html_label = <<~HTML
      <TABLE BORDER="0" CELLBORDER="0" CELLSPACING="0" CELLPADDING="4">
        <TR><TD COLSPAN="2" ALIGN="CENTER">#{label}</TD></TR>
        <TR><TD ALIGN="RIGHT">Curb:</TD><TD BGCOLOR="#FF9999" WIDTH="#{(curb_rps[i] * scale_factor).to_i}"> #{curb_rps[i].round(2)}</TD></TR>
        <TR><TD ALIGN="RIGHT">HTTP.rb:</TD><TD BGCOLOR="#99CCFF" WIDTH="#{(http_rps[i] * scale_factor).to_i}"> #{http_rps[i].round(2)}</TD></TR>
        <TR><TD ALIGN="RIGHT">Rquest-rb:</TD><TD BGCOLOR="#99FF99" WIDTH="#{(rquest_rps[i] * scale_factor).to_i}"> #{rquest_rps[i].round(2)}</TD></TR>
      </TABLE>
    HTML
    
    node = g.add_nodes("data#{i}", label: html_label)
    
    # Connect nodes in sequence
    g.add_edges(prev_node, node) if prev_node
    prev_node = node
  end
  
  # Add a legend
  legend_html = <<~HTML
    <TABLE BORDER="0" CELLBORDER="0" CELLSPACING="0" CELLPADDING="4">
      <TR><TD COLSPAN="2" ALIGN="CENTER"><B>Legend</B></TD></TR>
      <TR><TD BGCOLOR="#FF9999" WIDTH="20"></TD><TD>Curb</TD></TR>
      <TR><TD BGCOLOR="#99CCFF" WIDTH="20"></TD><TD>HTTP.rb</TD></TR>
      <TR><TD BGCOLOR="#99FF99" WIDTH="20"></TD><TD>Rquest-rb</TD></TR>
    </TABLE>
  HTML
  
  g.add_nodes("legend", label: legend_html)
end

# Save charts
time_graph.output(png: "#{output_dir}/time_chart.png")
time_graph.output(svg: "#{output_dir}/time_chart.svg")
rps_graph.output(png: "#{output_dir}/rps_chart.png")
rps_graph.output(svg: "#{output_dir}/rps_chart.svg")

puts "Generated benchmark charts:"
puts "- #{output_dir}/time_chart.png"
puts "- #{output_dir}/time_chart.svg"
puts "- #{output_dir}/rps_chart.png"
puts "- #{output_dir}/rps_chart.svg" 