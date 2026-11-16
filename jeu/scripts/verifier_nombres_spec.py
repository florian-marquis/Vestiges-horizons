#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Script jetable pour verifier tous les nombres annonces dans spec v1.1 et plan.
Calcule : valeur annoncee / valeur calculee / ecart
"""

print("=" * 80)
print("VERIFICATION DES NOMBRES DE LA SPEC V1.1")
print("=" * 80)

# 1. Pentes de G
print("\n--- 1. PENTES DE G (formule: pente = 100*u + 25*p - 30*q - 20000) ---")

test_cases = [
    ("u=100, p=100, q=100", 100, 100, 100),
    ("u=100, p=0, q=0", 100, 0, 0),
    ("u=200, p=0, q=0", 200, 0, 0),
    ("u=0, p=100, q=0", 0, 100, 0),
    ("u=0, p=0, q=100", 0, 0, 100),
]

for label, u, p, q in test_cases:
    pente = 100 * u + 25 * p - 30 * q - 20000
    print(f"  {label:25} -> {pente:8} (soulagement=20000)")

# 2. Pentes avec l'ancienne formule du code (pour comparaison)
print("\n--- 2. ANCIENNE FORMULE DU CODE (dg = (800*u + 200*p - 250*q) / 1000) ---")

for label, u, p, q in test_cases:
    dg = (800 * u + 200 * p - 250 * q) // 1000
    print(f"  {label:25} -> {dg:8}")

# 3. Valeurs de G apres mise a jour
print("\n--- 3. VALEURS DE G AVEC G_initial=30 (sans clamp) ---")

for label, u, p, q in test_cases:
    dg = (800 * u + 200 * p - 250 * q) // 1000
    g_final = 30 + dg
    print(f"  {label:25} -> G = 30 + {dg:3} = {g_final:5}")

# 4. Comparaison avec les valeurs annoncees dans le diagnostic
print("\n--- 4. VALEURS ANNONCEES DANS LE DIAGNOSTIC ---")
print("  Cohorte 1 (u=100,p=200,q=300): announced G=75, calculated dG=45, G_initial=30 -> 30+45=75 OK")
print("  Cohorte 2 (u=500,p=600,q=700): announced G=375, calculated dG=345, G_initial=30 -> 30+345=375 OK")

dg1 = (800*100 + 200*200 - 250*300) // 1000
dg2 = (800*500 + 200*600 - 250*700) // 1000
print(f"\n  dG1 = (800*100 + 200*200 - 250*300)/1000 = (80000+40000-75000)/1000 = 45000/1000 = {dg1}")
print(f"  dG2 = (800*500 + 200*600 - 250*700)/1000 = (400000+120000-175000)/1000 = 345000/1000 = {dg2}")
print(f"  Avec G_initial=30: G1=30+{dg1}={30+dg1}, G2=30+{dg2}={30+dg2}")

# 5. Seuil exponentiel
print("\n--- 5. SEUIL EXPONENTIEL (S = -ln(U) en entiers) ---")

import math

print("  Formule corrigee: S = max(1, (32*10^9 - L) * 252998721 / 10^9)")
print("  where L = round(log2(1 + mantisse/256) * 10^9)")
print("  252998721 = 365M * ln(2) / 10^9")
print(f"  Verification: 365000000 * ln(2) / 1000000000 = {365000000 * math.log(2) / 1000000000:.10f}")

# 6. Bandes de risque
print("\n--- 6. BANDES DE RISQUE (v1.1) ---")
print("  0 events: 0 units/jour")
print("  0.05/an: 0.05/365 units/jour = {:.10f}".format(0.05/365))
print("  0.2/an: 0.2/365 units/jour = {:.10f}".format(0.2/365))
print("  0.5/an: 0.5/365 units/jour = {:.10f}".format(0.5/365))

# 7. Echelle proposee
print("\n--- 7. ECHELLE PROPOSEE (1 evenement = 365000000 unites) ---")
print("  0.05/an = 0.05 * 365M / 365 = 50000 unites/jour")
print("  0.2/an = 0.2 * 365M / 365 = 200000 unites/jour")
print("  0.5/an = 0.5 * 365M / 365 = 500000 unites/jour")

# 8. Pentes de la spec v1.1
print("\n--- 8. PENTES DE LA SPEC V1.1 (corrigees) ---")
print("  pente = 100*u + 25*p - 30*q - 20000 [micro-pts/jour]")
print("  u, p, q sont des entiers dans [0; 1000]")
print("\n  Exemples:")
for u, p, q in [(100, 100, 100), (100, 0, 0), (200, 0, 0), (800, 0, 0)]:
    pente = 100 * u + 25 * p - 30 * q - 20000
    print(f"    u={u}, p={p}, q={q}: pente = 100*{u} + 25*{p} - 30*{q} - 20000 = {pente}")

# 9. Comparaison avec les valeurs annoncees
print("\n--- 9. VALEURS ANNONCEES vs CALCULEES ---")
print("  Format: Annoncee | Calculee | Ecart | Description")
print("  " + "-" * 60)

print(f"  {'+7500':>8} | {-10500:>8} | {-10500-7500:>8} | pente avec u=p=q=100")
print(f"  {'+8000':>8} | {-10000:>8} | {-10000-8000:>8} | pente avec u=100 seul")
print(f"  {'75':>8} | {75:>8} | {0:>8} | dG1 (u=100,p=200,q=300)")
print(f"  {'345':>8} | {345:>8} | {0:>8} | dG2 (u=500,p=600,q=700)")

print("\n" + "=" * 80)
print("CONCLUSION: Les pentes de la spec v1.1 sont FAUSSES")
print("  Pour que G monte avec p=q=0, il faut u > 200")
print("=" * 80)
