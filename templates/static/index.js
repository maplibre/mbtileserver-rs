import 'ol/ol.css';
import Map from 'ol/Map';
import Overlay from 'ol/Overlay';
import View from 'ol/View';
import * as proj from 'ol/proj';
import { MVT } from 'ol/format';
import { Tile as TileLayer, VectorTile as VectorTileLayer } from 'ol/layer';
import { OSM, UTFGrid, VectorTile as VectorTileSource, XYZ } from 'ol/source';
import stylefunction from 'ol-mapbox-style/src/stylefunction';
import { json } from 'd3-fetch';

const mapElement = document.getElementById('Map');

if (mapElement) {
  const raster = new TileLayer({
    source: new OSM()
  });

  const view = new View({
    minZoom: 0,
    maxZoom: 23,
    zoom: 0,
    multiWorld: true,
    showFullExtent: true
  });

  const map = new Map({
    layers: [raster],
    target: 'Map',
    view
  });

  json('./').then((tileJSON) => {
    if (tileJSON.maxzoom && tileJSON.maxzoom < view.getMaxZoom()) {
      view.setMaxZoom(tileJSON.maxzoom);
    }

    if (tileJSON.minzoom && tileJSON.minzoom > view.getMinZoom()) {
      view.setMinZoom(tileJSON.minzoom);
    }

    if (!tileJSON.bounds) {
      tileJSON.bounds = [-180, -85, 180, 85];
    }

    map.getView().fit(
      proj.transformExtent(tileJSON.bounds, proj.get('EPSG:4326'), proj.get('EPSG:3857')),
      {
        padding: [5, 5, 5, 5]
      }
    );

    let layer;
    if (tileJSON.format === 'pbf') {
      layer = new VectorTileLayer({
        declutter: true,
        source: new VectorTileSource({
          format: new MVT(),
          url: tileJSON.tiles[0]
        })
      });

      const styles = {
        version: 8,
        sources: {
          overlay: {
            type: 'vector',
            tiles: tileJSON.tiles,
            minzoom: tileJSON.minzoom,
            maxzoom: tileJSON.maxzoom
          }
        },
        layers: []
      }
      tileJSON.vector_layers.forEach((srcLyr, i) => {
        styles.layers.push({
          'id': `overlay-poly-' ${i}`,
          'source': 'overlay',
          'source-layer': srcLyr.id,
          'filter': ['==', '$type', 'Polygon'],
          'type': 'fill',
          'paint': {
            'fill-color': 'orange',
            'fill-opacity': 0.5,
            'fill-outline-color': 'red'
          }
        });

        styles.layers.push({
          'id': `overlay-line-' ${i}`,
          'source': 'overlay',
          'source-layer': srcLyr.id,
          'filter': ['==', '$type', 'LineString'],
          'type': 'line',
          'paint': {
            'line-color': 'red',
            'line-opacity': 0.75,
            'line-width': 2
          }
        });

        styles.layers.push({
          'id': `overlay-point-' ${i}`,
          'source': 'overlay',
          'source-layer': srcLyr.id,
          'filter': ['==', '$type', 'Point'],
          'type': 'circle',
          'paint': {
            'circle-radius': 6,
            'circle-color': '#F00',
            'circle-opacity': 1
          }
        });
      });

      stylefunction(
        layer,
        styles,
        'overlay'
      );
    } else {
      layer = new TileLayer({
        source: new XYZ({
          url: tileJSON.tiles[0],
          minZoom: tileJSON.minzoom || 0,
          maxZoom: tileJSON.maxzoom || 23
        })
      })
    }

    map.addLayer(layer);

    if (tileJSON.grids && tileJSON.grids.length > 0) {
      var gridSource = new UTFGrid({
        tileJSON
      });

      const gridLayer = new TileLayer({ source: gridSource });
      map.addLayer(gridLayer);

      const infoElement = document.getElementById('Info');

      infoOverlay = new Overlay({
        element: infoElement,
        offset: [15, 15],
        stopEvent: false
      });
      map.addOverlay(infoOverlay);

      const displayInfo = (coordinate) => {
        const viewResolution = view.getResolution();
        gridSource.forDataAtCoordinateAndResolution(
          coordinate,
          viewResolution,
          (data) => {
            mapElement.style.cursor = data ? 'pointer' : '';
            if (data) {
              infoElement.innerHTML = `
                  Data keys:
                  <br />
                  (see console for the full object)
                  <br />
                  ${JSON.stringify(Object.keys(data))}
                `
              console.log(data);
            }
            infoOverlay.setPosition(data ? coordinate : undefined);
          });
      };

      map.on('pointermove', (e) => {
        if (e.dragging) {
          return;
        }
        const coordinate = map.getEventCoordinate(e.originalEvent);
        displayInfo(coordinate);
      });
    }
  });
}