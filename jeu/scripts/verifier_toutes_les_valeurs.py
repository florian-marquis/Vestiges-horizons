#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Script pour verifier TOUTES les valeurs annonces dans spec v1.1 et plan.
Une ligne par nombre, avec formule et calcul.
"""

print("=" * 100)
print("TABLEAU COMPLET DES VALEURS ANNONCEES vs CALCULEES")
print("=" * 100)
print()

# Formules de reference
print("FORMULES DE REFERENCE:")
print("  Ancienne formule code: dg = (800*u + 200*p - 250*q) / 1000")
print("  Nouvelle formule spec v1.1: pente = 100*u + 25*p - 30*q - 20000 [micro-pts/jour]")
print("  Micro-pts: 1 G = 100 micro-pts (echelle proposee)")
print()

# Lignes du tableau
lignes = []

# 1. Pentes spec v1.1
lignes.append(("pente u=p=q=100", -10500, "100*100 + 25*100 - 30*100 - 20000", "spec v1.1"))
lignes.append(("pente u=100 seul", -10000, "100*100 + 25*0 - 30*0 - 20000", "spec v1.1"))
lignes.append(("pente u=200 seul", 0, "100*200 + 25*0 - 30*0 - 20000", "spec v1.1"))
lignes.append(("pente u=800 seul", 60000, "100*800 + 25*0 - 30*0 - 20000", "spec v1.1"))

# 2. dG avec ancienne formule
lignes.append(("dG1 (u=100,p=200,q=300)", 45, "(800*100 + 200*200 - 250*300) / 1000", "code actuel"))
lignes.append(("dG2 (u=500,p=600,q=700)", 345, "(800*500 + 200*600 - 250*700) / 1000", "code actuel"))

# 3. G avec G_initial=30
lignes.append(("G1 avec G_initial=30, dG=45", 75, "30 + 45", "code actuel"))
lignes.append(("G2 avec G_initial=30, dG=345", 375, "30 + 345", "code actuel"))
lignes.append(("G1 clampé", 75, "min(75, 100)", "code actuel"))
lignes.append(("G2 clampé", 100, "min(375, 100)", "code actuel"))

# 4. Valeurs du diagnostic v3 (annoncees)
# L'utilisateur dit que v3 annonçait G=75 et G=375
lignes.append(("G1 annoncé diagnostic v3", 75, "30 + 45", "diagnostic v3"))
lignes.append(("G2 annoncé diagnostic v3", 375, "30 + 345", "diagnostic v3"))

# 5. Valeurs spec v1.1 (tests A1, B3, etc.) - a verifier
# L'utilisateur mentionne "75 000" et "3 750 jours"
# 75 000 micro-pts = 75 G
lignes.append(("75 000 micro-pts", 75000, "75 * 1000", "echelle micro-pts"))
lignes.append(("75 000 micro-pts en G", 75, "75000 / 1000", "conversion"))

# 6. Valeurs de temps
# "3 750 jours" - a vérifier
# Avec pente = 100 u, et u=800 (penurie durable)
# pente = 100*800 - 20000 = 60000 micro-pts/jour
# Pour atteindre seuil de 75 G = 75000 micro-pts:
# jours = ceil(75000 / 60000) = 2 jours
# Pas 3750...
# Peut-être avec une autre échelle ?

# 7. Bandes de risque
lignes.append(("Bande G<30: lambda", 0, "0 events/jour", "spec v1.1"))
lignes.append(("Bande 30-60: lambda", 50000, "0.05/an * 365M/365", "spec v1.1"))
lignes.append(("Bande 60-80: lambda", 200000, "0.2/an * 365M/365", "spec v1.1"))
lignes.append(("Bande >80: lambda", 500000, "0.5/an * 365M/365", "spec v1.1"))

# 8. Seuil exponentiel
import math
lignes.append(("ln(2)", 0.69314718056, "math.log(2)", "constante"))
lignes.append(("365M * ln(2) / 10^9", 252998720.9, "365000000 * ln(2) / 1000000000", "spec v1.1"))
lignes.append(("365M * ln(2) / 10^9 arrondi", 252998721, "round(365000000 * ln(2) / 1000000000)", "spec v1.1"))

# Affichage
print("Format: Description | Valeur Calculee | Formule | Source")
print("-" * 100)

for desc, valeur, formule, source in lignes:
    print(f"  {desc:35} | {valeur:>18} | {formule:30} | {source}")

print()
print("=" * 100)
print("NOTES:")
print("  - Les pentes de spec v1.1 avec soulagement sont NEGATIVES pour u<=200")
print("  - Pour avoir G > 0, il faut u > 200 (avec p=q=0)")
print("  - Les valeurs 75 et 375 du diagnostic v3 correspondent a G_initial=30 + dG")
print("  - Le clamp a 100 explique pourquoi G2=100 dans les tests")
print("  - 75 000 micro-pts = 75 G (si 1 G = 1000 micro-pts) ou 750 G (si 1 G = 100 micro-pts)")
print("=" * 100)
