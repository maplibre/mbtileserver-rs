import "ol/ol.css";
import Map from "ol/Map";
import Overlay from "ol/Overlay";
import View from "ol/View";
import * as proj from "ol/proj";
import { MVT } from "ol/format";
import { Tile as TileLayer, VectorTile as VectorTileLayer } from "ol/layer";
import { OSM, UTFGrid, VectorTile as VectorTileSource, XYZ } from "ol/source";
import stylefunction from "ol-mapbox-style/src/stylefunction";

import L from "leaflet";
import "leaflet/dist/leaflet.css";
import "leaflet-zoombox";
import "leaflet-range";
import "leaflet-basemaps";

import { json } from "d3-fetch";

var map = window.location.search.substring(1);
if (map != "leaflet" && map != "ol") map = "ol";

if (map == "leaflet") {
  var basemaps = [
    L.tileLayer(
      "//{s}.arcgisonline.com/ArcGIS/rest/services/World_Topo_Map/MapServer/tile/{z}/{y}/{x}",
      {
        attribution:
          "Tiles &copy; Esri &mdash; Esri, DeLorme, NAVTEQ, TomTom, Intermap, iPC, USGS, FAO, NPS, NRCAN, GeoBase, Kadaster NL, Ordnance Survey, Esri Japan, METI, Esri China (Hong Kong), and the GIS User Community",
        subdomains: ["server", "services"],
        label: "ESRI Topo",
      }
    ),
    L.tileLayer(
      "//server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}",
      {
        attribution:
          "Tiles &copy; Esri &mdash; Source: Esri, i-cubed, USDA, USGS, AEX, GeoEye, Getmapping, Aerogrid, IGN, IGP, UPR-EGP, and the GIS User Community",
        label: "ESRI Imagery",
      }
    ),
    L.tileLayer(
      "//{s}.arcgisonline.com/ArcGIS/rest/services/Canvas/World_Light_Gray_Base/MapServer/tile/{z}/{y}/{x}",
      {
        attribution: "Tiles &copy; Esri &mdash; Esri, DeLorme, NAVTEQ",
        maxZoom: 16,
        subdomains: ["server", "services"],
        label: "ESRI Gray",
      }
    ),
    L.tileLayer(
      "//{s}.arcgisonline.com/arcgis/rest/services/Elevation/World_Hillshade/MapServer/tile/{z}/{y}/{x}",
      {
        attribution:
          "Esri, USGS, NGA, NASA, CGIAR, N Robinson, NCEAS, NLS, OS, NMA, Geodatastyrelsen, Rijkswaterstaat, GSA, Geoland, FEMA, Intermap and the GIS user community",
        maxZoom: 23,
        subdomains: ["server", "services"],
        label: "ESRI Elevation",
      }
    ),
  ];

  var map = L.map("Map", {
    preferCanvas: true,
  });

  var layer = null;
  json("./").then(function (tileJSON) {
    if (tileJSON.bounds) {
      var b = tileJSON.bounds;

      map.fitBounds([
        [b[1], b[0]],
        [b[3], b[2]],
      ]);
    } else {
      map.fitWorld();
    }

    if (tileJSON.maxzoom && tileJSON.maxzoom < map.getZoom()) {
      map.setZoom(tileJSON.maxzoom);
    }

    if (tileJSON.format === "pbf") {
      var vectorTileLayerStyles = {};
      tileJSON.vector_layers.forEach(function (vl) {
        vectorTileLayerStyles[vl.id] = function () {
          return [
            { radius: 6, opacity: 1, color: "#F00" }, // point
            { width: 2, opacity: 0.75, color: "red" }, // line
            { fill: true, opacity: 0.5, fillColor: "orange" }, // poly
          ];
        };
      });
      layer = L.vectorGrid.protobuf(tileJSON.tiles[0], {
        rendererFactory: L.canvas.tile,
        vectorTileLayerStyles: vectorTileLayerStyles,
      });
    } else {
      layer = L.tileLayer(tileJSON.tiles[0], {
        minZoom: tileJSON.minzoom || 0,
        maxZoom: tileJSON.maxzoom || 23,
      });
    }

    map.addLayer(layer);

    var legendJSON = tileJSON.legend;
    if (legendJSON && legendJSON.search(/\{/) === 0) {
      legendJSON = JSON.parse(legendJSON);

      if (legendJSON.length && legendJSON[0].elements) {
        map.addControl(
          L.control.base64legend({
            position: "topright",
            legends: legendJSON,
            collapseSimple: true,
            detectStretched: true,
          })
        );
      }
    }

    if (tileJSON.grids && tileJSON.grids.length > 0) {
      var gridURL = tileJSON.grids[0];
      var utfgrid = L.utfGrid(gridURL, {
        resolution: 4,
        pointerCursor: true,
        mouseInterval: 66,
      });
      utfgrid.addTo(map);

      var infoContainer = L.DomUtil.create("div", "info", L.DomUtil.get("Map"));
      var textNode = L.DomUtil.create("h3", "", infoContainer);
      textNode.innerHTML =
        "This service has UTF-8 Grids; see console for grid data.";

      utfgrid.on("mouseover", function (e) {
        console.log("UTF grid data:", e.data);
      });
    }
  });

  map.zoomControl.setPosition("topleft");
  map.addControl(L.control.zoomBox({ modal: true, position: "topleft" }));

  var slider = L.control.range({
    position: "topleft",
    min: 0,
    max: 1,
    value: 1,
    step: 0.01,
    orient: "vertical",
    iconClass: "leaflet-range-icon",
  });

  slider.on("input change", function (e) {
    layer.setOpacity(e.value);
  });

  map.addControl(slider);

  map.addControl(
    L.control.basemaps({
      position: "bottomright",
      basemaps: basemaps,
      tileX: 0,
      tileY: 0,
      tileZ: 1,
    })
  );
} else if (map == "ol") {
  const mapElement = document.getElementById("Map");

  if (mapElement) {
    const raster = new TileLayer({
      source: new OSM(),
    });

    const view = new View({
      minZoom: 0,
      maxZoom: 23,
      zoom: 0,
      multiWorld: true,
      showFullExtent: true,
    });

    const map = new Map({
      layers: [raster],
      target: "Map",
      view,
    });

    json("./").then((tileJSON) => {
      if (tileJSON.maxzoom && tileJSON.maxzoom < view.getMaxZoom()) {
        view.setMaxZoom(tileJSON.maxzoom);
      }

      if (tileJSON.minzoom && tileJSON.minzoom > view.getMinZoom()) {
        view.setMinZoom(tileJSON.minzoom);
      }

      if (!tileJSON.bounds) {
        tileJSON.bounds = [-180, -85, 180, 85];
      }

      map
        .getView()
        .fit(
          proj.transformExtent(
            tileJSON.bounds,
            proj.get("EPSG:4326"),
            proj.get("EPSG:3857")
          ),
          {
            padding: [5, 5, 5, 5],
          }
        );

      let layer;
      if (tileJSON.format === "pbf") {
        layer = new VectorTileLayer({
          declutter: true,
          source: new VectorTileSource({
            format: new MVT(),
            url: tileJSON.tiles[0],
          }),
        });

        const styles = {
          version: 8,
          sources: {
            overlay: {
              type: "vector",
              tiles: tileJSON.tiles,
              minzoom: tileJSON.minzoom,
              maxzoom: tileJSON.maxzoom,
            },
          },
          layers: [],
        };
        tileJSON.vector_layers.forEach((srcLyr, i) => {
          styles.layers.push({
            id: `overlay-poly-' ${i}`,
            source: "overlay",
            "source-layer": srcLyr.id,
            filter: ["==", "$type", "Polygon"],
            type: "fill",
            paint: {
              "fill-color": "orange",
              "fill-opacity": 0.5,
              "fill-outline-color": "red",
            },
          });

          styles.layers.push({
            id: `overlay-line-' ${i}`,
            source: "overlay",
            "source-layer": srcLyr.id,
            filter: ["==", "$type", "LineString"],
            type: "line",
            paint: {
              "line-color": "red",
              "line-opacity": 0.75,
              "line-width": 2,
            },
          });

          styles.layers.push({
            id: `overlay-point-' ${i}`,
            source: "overlay",
            "source-layer": srcLyr.id,
            filter: ["==", "$type", "Point"],
            type: "circle",
            paint: {
              "circle-radius": 6,
              "circle-color": "#F00",
              "circle-opacity": 1,
            },
          });
        });

        stylefunction(layer, styles, "overlay");
      } else {
        layer = new TileLayer({
          source: new XYZ({
            url: tileJSON.tiles[0],
            minZoom: tileJSON.minzoom || 0,
            maxZoom: tileJSON.maxzoom || 23,
          }),
        });
      }

      map.addLayer(layer);

      if (tileJSON.grids && tileJSON.grids.length > 0) {
        var gridSource = new UTFGrid({
          tileJSON,
        });

        const gridLayer = new TileLayer({ source: gridSource });
        map.addLayer(gridLayer);

        const infoElement = document.getElementById("Info");

        infoOverlay = new Overlay({
          element: infoElement,
          offset: [15, 15],
          stopEvent: false,
        });
        map.addOverlay(infoOverlay);

        const displayInfo = (coordinate) => {
          const viewResolution = view.getResolution();
          gridSource.forDataAtCoordinateAndResolution(
            coordinate,
            viewResolution,
            (data) => {
              mapElement.style.cursor = data ? "pointer" : "";
              if (data) {
                infoElement.innerHTML = `
                  Data keys:
                  <br />
                  (see console for the full object)
                  <br />
                  ${JSON.stringify(Object.keys(data))}
                `;
                console.log(data);
              }
              infoOverlay.setPosition(data ? coordinate : undefined);
            }
          );
        };

        map.on("pointermove", (e) => {
          if (e.dragging) {
            return;
          }
          const coordinate = map.getEventCoordinate(e.originalEvent);
          displayInfo(coordinate);
        });
      }
    });
  }
}
